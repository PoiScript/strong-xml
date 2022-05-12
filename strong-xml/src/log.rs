#[macro_export]
#[doc(hidden)]
macro_rules! log_start_reading {
    ($element:path) => {
        $crate::lib::log::debug!(concat!("[", stringify!($element), "] Started reading"));
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_finish_reading {
    ($element:path) => {
        $crate::lib::log::debug!(concat!("[", stringify!($element), "] Finished reading"));
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_start_reading_field {
    ($element:path, $name:expr) => {
        $crate::lib::log::trace!(concat!(
            "[",
            stringify!($element),
            "] Started reading field `",
            stringify!($name),
            "`"
        ));
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_finish_reading_field {
    ($element:path, $name:expr) => {
        $crate::lib::log::trace!(concat!(
            "[",
            stringify!($element),
            "] Finished reading field `",
            stringify!($name),
            "`"
        ));
    };
}


#[macro_export]
#[doc(hidden)]
macro_rules! make_tag {
    ($prefix:ident, $local:ident) => {
        if $prefix.is_empty() {
            $local.to_owned()
        } else {
            ($prefix.to_owned() + ":" + $local)
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_skip_attribute {
    ($element:path, $key:ident) => {
        $crate::lib::log::info!(
            concat!("[", stringify!($element), "] Skip attribute `{}`"),
            $key
        );
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_skip_element {
    ($element:path, $name:ident) => {
        $crate::lib::log::info!(
            concat!("[", stringify!($element), "] Skip element `{}`"),
            $name
        );
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_start_writing {
    ($element:path) => {
        $crate::lib::log::debug!(concat!("[", stringify!($element), "] Started writing"));
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_finish_writing {
    ($element:path) => {
        $crate::lib::log::debug!(concat!("[", stringify!($element), "] Finished writing"));
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_start_writing_field {
    ($element:path, $name:expr) => {
        $crate::lib::log::trace!(concat!(
            "[",
            stringify!($element),
            "] Started writing field `",
            stringify!($name),
            "`"
        ));
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! log_finish_writing_field {
    ($element:path, $name:expr) => {
        $crate::lib::log::trace!(concat!(
            "[",
            stringify!($element),
            "] Finished writing field `",
            stringify!($name),
            "`"
        ));
    };
}
