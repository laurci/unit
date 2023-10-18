extern "C" {
    pub fn unit_log(ptr: i32, len: i32);
    pub fn unit_send_message(ptr: i32, len: i32);

    // pub fn unit_save_shared_object(index: i32, ptr: i32, len: i32);
    // pub fn unit_lock_shared_object(index: i32);
    // pub fn unit_unlock_shared_object(index: i32);
    // pub fn unit_get_shared_object_len(index: i32) -> i32;
    // pub fn unit_load_shared_object(index: i32, ptr: i32, len: i32);
}
