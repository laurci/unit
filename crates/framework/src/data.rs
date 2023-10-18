#[macro_export]
macro_rules! data {
    { $vis:vis $name:ident:$type:ty = $initializer:expr } => {
        $vis fn $name() -> &'static $type {
            static mut SINGLETON: std::mem::MaybeUninit<$type> = std::mem::MaybeUninit::uninit();
            static ONCE: std::sync::Once = std::sync::Once::new();

            unsafe {
                ONCE.call_once(|| {
                    let singleton = $initializer;
                    SINGLETON.write(singleton);
                });

                SINGLETON.assume_init_ref()
            }
        }
    };
}
