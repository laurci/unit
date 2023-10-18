use crate::bus::{Bus, BusMessage};
use axum::extract::ws::Message;
use log::info;
use unit_abi::header::AbiHeader;
use unit_runtime_proto::{
    decode_runtime_proto_message, encode_runtime_proto_message, CrossbarMessage, WsMessage,
};
use unit_utils::Result;
use wasmer::{
    imports, Function, FunctionEnv, FunctionEnvMut, Imports, Instance, Memory, Module, Store,
    StoreMut, Value,
};
use wasmer_wasix::{WasiEnv, WasiFunctionEnv};

#[derive(Clone)]
pub struct RuntimeEnv {
    pub connection_id: String,
    pub memory: Option<Memory>,
    pub bus: Bus,
}

impl RuntimeEnv {
    pub fn new(connection_id: String, bus: Bus) -> Self {
        Self {
            memory: None,
            connection_id,
            bus,
        }
    }

    pub fn initialize(&mut self, memory: Memory) {
        self.memory = Some(memory);
    }

    pub fn read_memory(&self, store: &StoreMut, ptr: i32, len: i32) -> Vec<u8> {
        let memory = self.memory.as_ref().unwrap();
        let mut bytes_vec = vec![0 as u8; len as usize];
        memory.view(store).read(ptr as _, &mut bytes_vec).unwrap();
        bytes_vec
    }

    #[allow(unused)]
    pub fn write_memory(&self, store: &StoreMut, ptr: i32, bytes: &[u8]) {
        let memory = self.memory.as_ref().unwrap();
        memory.view(store).write(ptr as _, bytes).unwrap();
    }
}

pub struct Runtime {
    pub connection_id: String,
    pub app_name: String,
    pub app_path: String,
    pub abi_header: AbiHeader,
    pub store: Store,
    pub module: Module,
    pub memory: Memory,
    pub runtime_env: RuntimeEnv,
    pub runtime_env_instance: FunctionEnv<RuntimeEnv>,
    pub wasi_env: WasiFunctionEnv,
    pub imports: Imports,
    pub instance: Instance,
}

fn unit_log(mut unit_env: FunctionEnvMut<RuntimeEnv>, ptr: i32, len: i32) {
    let (env, store) = unit_env.data_and_store_mut();
    let bytes = env.read_memory(&store, ptr, len);
    let s = String::from_utf8(bytes.to_owned()).unwrap();
    info!("[{}][log]: {}", env.connection_id, s);
}

fn unit_send_message(mut unit_env: FunctionEnvMut<RuntimeEnv>, ptr: i32, len: i32) {
    let (env, store) = unit_env.data_and_store_mut();
    let bytes = env.read_memory(&store, ptr, len);
    let message: WsMessage = decode_runtime_proto_message(bytes).unwrap();
    let message: Message = match message {
        WsMessage::Text(text) => Message::Text(text),
        WsMessage::Binary(binary) => Message::Binary(binary),
    };

    env.bus.send(BusMessage::TxWsMessage {
        connection_id: env.connection_id.clone(),
        message,
    });
}

impl Runtime {
    pub fn new(
        app_name: String,
        app_path: String,
        header: AbiHeader,
        mut runtime_env: RuntimeEnv,
    ) -> Result<Self> {
        let app_bytes = std::fs::read(app_path.clone())?;

        let mut store = Store::default();
        let module = Module::new(&store, app_bytes)?;

        let memory_ty = module.imports().memories().next().map(|a| *a.ty()).unwrap();
        let memory = Memory::new(&mut store, memory_ty)?;

        runtime_env.initialize(memory.clone());

        let runtime_env_instance = FunctionEnv::new(&mut store, runtime_env.clone());
        let mut wasi_env = WasiEnv::builder(app_name.clone()).finalize(&mut store)?;

        let lib_imports = imports! {
            "env" => {
                "unit_log" => Function::new_typed_with_env(&mut store, &runtime_env_instance, unit_log),
                "unit_send_message" => Function::new_typed_with_env(&mut store, &runtime_env_instance, unit_send_message),
                // "unit_save_shared_object" => Function::new_typed_with_env(&mut store, &unit_env_instance, unit_save_shared_object),
                // "unit_lock_shared_object" => Function::new_typed_with_env(&mut store, &unit_env_instance, unit_lock_shared_object),
                // "unit_unlock_shared_object" => Function::new_typed_with_env(&mut store, &unit_env_instance, unit_unlock_shared_object),
                // "unit_get_shared_object_len" => Function::new_typed_with_env(&mut store, &unit_env_instance, unit_get_shared_object_len),
                // "unit_load_shared_object" => Function::new_typed_with_env(&mut store, &unit_env_instance, unit_load_shared_object),
            }
        };

        let mut import_object =
            wasi_env.import_object_for_all_wasi_versions(&mut store, &module)?;
        import_object.define("env", "memory", memory.clone());

        let thread_spawn = import_object
            .get_export("wasi_snapshot_preview1", "thread-spawn")
            .unwrap();
        import_object.define("wasi", "thread-spawn", thread_spawn);

        import_object.extend(lib_imports.into_iter());

        let instance = Instance::new(&mut store, &module, &import_object)?;

        wasi_env.initialize_with_memory(
            &mut store,
            instance.clone(),
            Some(memory.clone()),
            true,
        )?;

        Ok(Self {
            connection_id: runtime_env.connection_id.clone(),
            app_name,
            app_path,
            abi_header: header,
            store,
            module,
            memory,
            runtime_env,
            runtime_env_instance,
            wasi_env,
            imports: import_object,
            instance,
        })
    }

    fn call_fn(&mut self, name: &str, args: &[Value]) -> Result<Vec<Value>> {
        let fn_ = self.instance.exports.get_function(name).unwrap();
        let results = fn_.call(&mut self.store, args).unwrap();

        Ok(Vec::from(results))
    }

    fn call_fn_if_exists(&mut self, name: &str, args: &[Value]) -> Result<Option<Vec<Value>>> {
        let fn_ = self.instance.exports.get_function(name);

        if fn_.is_err() {
            return Ok(None);
        }
        let fn_ = fn_.unwrap();

        let results = fn_.call(&mut self.store, args).unwrap();

        Ok(Some(Vec::from(results)))
    }

    fn alloc_bytes(&mut self, len: usize) -> Result<usize> {
        let ptr =
            self.call_fn("unit_alloc_bytes", &[Value::I32(len as i32)])?[0].unwrap_i32() as usize;
        Ok(ptr)
    }

    // fn free_bytes(&mut self, ptr: usize, len: usize) -> Result<()> {
    //     self.call_fn(
    //         "unit_free_bytes",
    //         &[Value::I32(ptr as i32), Value::I32(len as i32)],
    //     )?;
    //
    //     Ok(())
    // }

    fn write_mem(&mut self, ptr: usize, bytes: &[u8]) -> Result<()> {
        let view = &self.memory.view(&self.store);
        for (i, byte) in bytes.iter().enumerate() {
            view.write_u8((ptr + i) as _, *byte).unwrap();
        }

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        self.wasi_env.data(&self.store).thread.set_status_running();

        self.call_fn("_start", &[])?;

        self.call_fn_if_exists("unit_init", &[])?;

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.call_fn_if_exists("unit_cleanup", &[])?;

        // pub fn message(&mut self, )
        Ok(())
    }

    pub fn message(&mut self, msg: WsMessage) -> Result<()> {
        let message = encode_runtime_proto_message(&msg)?;
        let ptr = self.alloc_bytes(message.len())?;
        self.write_mem(ptr, &message)?;

        self.call_fn_if_exists(
            "unit_message",
            &[Value::I32(ptr as i32), Value::I32(message.len() as i32)],
        )?;

        Ok(())
    }

    pub fn crossbar_event(&mut self, event: CrossbarMessage) -> Result<()> {
        let encoded_event = encode_runtime_proto_message(&event)?;
        let ptr = self.alloc_bytes(encoded_event.len())?;
        self.write_mem(ptr, &encoded_event)?;

        let normalized_event_fn_name = format!(
            "unit_topic_{}",
            event.topic.to_lowercase().replace("-", "_")
        );

        let res = self.call_fn_if_exists(
            &normalized_event_fn_name,
            &[
                Value::I32(ptr as i32),
                Value::I32(encoded_event.len() as i32),
            ],
        )?;

        // we have a specialized handler for this topic
        if res.is_some() {
            return Ok(());
        }

        self.call_fn_if_exists(
            "unit_event",
            &[
                Value::I32(ptr as i32),
                Value::I32(encoded_event.len() as i32),
            ],
        )?;

        Ok(())
    }
}
