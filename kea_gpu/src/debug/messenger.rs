use ash::{extensions::ext, vk};
use std::{ffi::CStr, os::raw::c_void};

unsafe extern "system" fn vulkan_debug_callback(
    flag: vk::DebugUtilsMessageSeverityFlagsEXT,
    typ: vk::DebugUtilsMessageTypeFlagsEXT,
    callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*callback_data).p_message);
    match flag {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => log::debug!("{:?} - {:?}", typ, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => log::info!("{:?} - {:?}", typ, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => log::warn!("{:?} - {:?}", typ, message),
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => log::error!("{:?} - {:?}", typ, message),
        _ => {
            log::error!("Unknown message severity flag {:?}", flag);
            log::error!("{:?} - {:?}", typ, message);
        }
    }

    vk::FALSE
}

pub struct DebugMessenger(vk::DebugUtilsMessengerEXT);

impl DebugMessenger {
    pub fn new(ext: &ext::DebugUtils) -> Self {
        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                    | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let messenger = unsafe {
            ext.create_debug_utils_messenger(&create_info, None)
                .unwrap()
        };

        Self(messenger)
    }
}
