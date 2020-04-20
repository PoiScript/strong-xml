#[macro_export]
#[doc(hidden)]
macro_rules! log_start_reading {
    ($element:path) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_finish_reading {
    ($element:path) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_start_reading_field {
    ($element:path, $name:ident) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_finish_reading_field {
    ($element:path, $name:ident) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_skip_attribute {
    ($element:path, $key:ident) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_skip_element {
    ($element:path, $tag:ident) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_start_writing {
    ($element:path) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_finish_writing {
    ($element:path) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_start_writing_field {
    ($element:path, $name:ident) => {};
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_finish_writing_field {
    ($element:path, $name:ident) => {};
}
