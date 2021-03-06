use cef_sys::{
    cef_post_data_create, cef_post_data_element_create, cef_post_data_element_t, cef_post_data_t,
    cef_postdataelement_type_t, cef_referrer_policy_t, cef_request_create, cef_request_t,
    cef_resource_type_t, cef_string_userfree_utf16_free,
};
use std::{collections::HashMap, convert::TryFrom, ptr::null_mut};

use crate::{load_handler::TransitionType, multimap::MultiMap, string::CefString};

/// Policy for how the Referrer HTTP header value will be sent during navigation.
/// if the `--no-referrers` command-line flag is specified then the policy value
/// will be ignored and the Referrer value will never be sent.
/// Must be kept synchronized with `net::URLRequest::ReferrerPolicy` from Chromium.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReferrerPolicy {
    /// Clear the referrer header if the header value is HTTPS but the request
    /// destination is HTTP. This is the default behavior.1
    Default = cef_referrer_policy_t::REFERRER_POLICY_DEFAULT as isize,
    /// A slight variant on CLEAR_REFERRER_ON_TRANSITION_FROM_SECURE_TO_INSECURE (Default):
    /// if the request destination is HTTP, an HTTPS referrer will be cleared. if
    /// the request's destination is cross-origin with the referrer (but does not
    /// downgrade), the referrer's granularity will be stripped down to an origin
    /// rather than a full URL. Same-origin requests will send the full referrer.
    ReduceReferrerGranularityOnTransitionCrossOrigin = cef_referrer_policy_t::REFERRER_POLICY_REDUCE_REFERRER_GRANULARITY_ON_TRANSITION_CROSS_ORIGIN as isize,
    /// Strip the referrer down to an origin when the origin of the referrer is
    /// different from the destination's origin.
    OriginOnlyOnTransitionCrossOrigin = cef_referrer_policy_t::REFERRER_POLICY_ORIGIN_ONLY_ON_TRANSITION_CROSS_ORIGIN as isize,
    /// Never change the referrer.
    NeverClearReferrer = cef_referrer_policy_t::REFERRER_POLICY_NEVER_CLEAR_REFERRER as isize,
    /// Strip the referrer down to the origin regardless of the redirect location.
    Origin = cef_referrer_policy_t::REFERRER_POLICY_ORIGIN as isize,
    /// Clear the referrer when the request's referrer is cross-origin with the
    /// request's destination.
    ClearReferrerOnTransitionCrossOrigin = cef_referrer_policy_t::REFERRER_POLICY_CLEAR_REFERRER_ON_TRANSITION_CROSS_ORIGIN as isize,
    /// Strip the referrer down to the origin, but clear it entirely if the
    /// referrer value is HTTPS and the destination is HTTP.
    OriginClearOnTransitionFromSecureToInsecure = cef_referrer_policy_t::REFERRER_POLICY_ORIGIN_CLEAR_ON_TRANSITION_FROM_SECURE_TO_INSECURE as isize,
    /// Always clear the referrer regardless of the request destination.
    NoReferrer = cef_referrer_policy_t::REFERRER_POLICY_NO_REFERRER as isize,
}

impl ReferrerPolicy {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

/// Flags used to customize the behavior of [URLRequest].
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum URLRequestFlags {
    /// if set the cache will be skipped when handling the request. Setting this
    /// value is equivalent to specifying the "Cache-Control: no-cache" request
    /// header. Setting this value in combination with OnlyFromCache will
    /// cause the request to fail.
    SkipCache = 0,
    /// if set the request will fail if it cannot be served from the cache (or some
    /// equivalent local store). Setting this value is equivalent to specifying the
    /// "Cache-Control: only-if-cached" request header. Setting this value in
    /// combination with SkipCache or DisableCache will cause the
    /// request to fail.
    OnlyFromCache = 1,
    /// if set the cache will not be used at all. Setting this value is equivalent
    /// to specifying the "Cache-Control: no-store" request header. Setting this
    /// value in combination with OnlyFromCache will cause the request to
    /// fail.
    DisableCache = 2,
    /// if set user name, password, and cookies may be sent with the request, and
    /// cookies may be saved from the response.
    AllowStoredCredentials = 3,
    /// if set upload progress events will be generated when a request has a body.
    ReportUploadProgress = 4,
    /// if set the [URLRequestClientCallbacks::on_download_data] method will not be called.
    NoDownloadData = 5,
    /// if set 5XX redirect errors will be propagated to the observer instead of
    /// automatically re-tried. This currently only applies for requests
    /// originated in the browser process.
    NoRetryOn5xx = 6,
    /// if set 3XX responses will cause the fetch to halt immediately rather than
    /// continue through the redirect.
    StopOnRedirect = 7,
}

impl URLRequestFlags {
    pub(crate) fn to_bitfield(flags: &[URLRequestFlags]) -> i32 {
        flags
            .iter()
            .fold(0, |flags, flag| flags | (1 << (*flag) as i32))
    }
    pub(crate) fn from_bitfield(bitfield: i32) -> Vec<URLRequestFlags> {
        [
            URLRequestFlags::SkipCache,
            URLRequestFlags::OnlyFromCache,
            URLRequestFlags::DisableCache,
            URLRequestFlags::AllowStoredCredentials,
            URLRequestFlags::ReportUploadProgress,
            URLRequestFlags::NoDownloadData,
            URLRequestFlags::NoRetryOn5xx,
            URLRequestFlags::StopOnRedirect,
        ]
        .iter()
        .filter(|flag| bitfield & (1 << (**flag) as i32) != 0)
        .cloned()
        .collect()
    }
}

/// Resource type for a request.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    /// Top level page.
    MainFrame = cef_resource_type_t::RT_MAIN_FRAME as isize,
    /// Frame or iframe.
    SubFrame = cef_resource_type_t::RT_SUB_FRAME as isize,
    /// CSS stylesheet.
    Stylesheet = cef_resource_type_t::RT_STYLESHEET as isize,
    /// External script.
    Script = cef_resource_type_t::RT_SCRIPT as isize,
    /// Image (jpg/gif/png/etc).
    Image = cef_resource_type_t::RT_IMAGE as isize,
    /// Font.
    FontResource = cef_resource_type_t::RT_FONT_RESOURCE as isize,
    /// Some other subresource. This is the default type if the actual type is
    /// unknown.
    SubResource = cef_resource_type_t::RT_SUB_RESOURCE as isize,
    /// Object (or embed) tag for a plugin, or a resource that a plugin requested.
    Object = cef_resource_type_t::RT_OBJECT as isize,
    /// Media resource.
    Media = cef_resource_type_t::RT_MEDIA as isize,
    /// Main resource of a dedicated worker.
    Worker = cef_resource_type_t::RT_WORKER as isize,
    /// Main resource of a shared worker.
    SharedWorker = cef_resource_type_t::RT_SHARED_WORKER as isize,
    /// Explicitly requested prefetch.
    Prefetch = cef_resource_type_t::RT_PREFETCH as isize,
    /// Favicon.
    Favicon = cef_resource_type_t::RT_FAVICON as isize,
    /// XMLHttpRequest.
    XHR = cef_resource_type_t::RT_XHR as isize,
    /// A request for a <ping>
    Ping = cef_resource_type_t::RT_PING as isize,
    /// Main resource of a service worker.
    ServiceWorker = cef_resource_type_t::RT_SERVICE_WORKER as isize,
    /// A report of Content Security Policy violations.
    CSPReport = cef_resource_type_t::RT_CSP_REPORT as isize,
    /// A resource that a plugin requested.
    PluginResource = cef_resource_type_t::RT_PLUGIN_RESOURCE as isize,
}

impl ResourceType {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PostDataElementType {
    Empty = cef_postdataelement_type_t::PDE_TYPE_EMPTY as isize,
    Bytes = cef_postdataelement_type_t::PDE_TYPE_BYTES as isize,
    File = cef_postdataelement_type_t::PDE_TYPE_FILE as isize,
}

impl PostDataElementType {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

ref_counted_ptr! {
    /// Structure used to represent a web request. The functions of this structure
    /// may be called on any thread.
    pub struct Request(*mut cef_request_t);
}

impl Request {
    /// Create a new Request object.
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_request_create()) }
    }

    /// Returns true if this object is read-only.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.0.as_ptr()) != 0 })
            .unwrap_or(true)
    }
    /// Get the fully qualified URL.
    pub fn get_url(&self) -> String {
        self.0
            .get_url
            .and_then(|get_url| unsafe { get_url(self.as_ptr()).as_mut() })
            .map(|url| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(url));
                cef_string_userfree_utf16_free(url);
                s
            })
            .unwrap_or_default()
    }
    /// Set the fully qualified URL.
    pub fn set_url(&self, url: &str) {
        if let Some(set_url) = self.0.set_url {
            unsafe {
                set_url(self.0.as_ptr(), CefString::new(url).as_ptr());
            }
        }
    }
    /// Get the request function type. The value will default to POST if post data
    /// is provided and GET otherwise.
    pub fn get_method(&self) -> String {
        self.0
            .get_method
            .and_then(|get_method| unsafe { get_method(self.as_ptr()).as_mut() })
            .map(|cef_string| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(cef_string));
                cef_string_userfree_utf16_free(cef_string);
                s
            })
            .unwrap_or_else(|| "GET".to_owned())
    }
    /// Set the request function type.
    pub fn set_method(&self, method: &str) {
        if let Some(set_method) = self.0.set_method {
            unsafe {
                set_method(self.0.as_ptr(), CefString::new(method).as_ptr());
            }
        }
    }
    /// Set the referrer URL and policy. if `Some` the referrer URL must be fully
    /// qualified with an HTTP or HTTPS scheme component. Any username, password or
    /// ref component will be removed.
    pub fn set_referrer(&self, referrer_url: Option<&str>, policy: ReferrerPolicy) {
        if let Some(set_referrer) = self.0.set_referrer {
            if let Some(referrer_url) = referrer_url {
                unsafe {
                    set_referrer(
                        self.0.as_ptr(),
                        CefString::new(referrer_url).as_ptr(),
                        policy as cef_referrer_policy_t::Type,
                    );
                }
            }
        }
    }
    /// Get the referrer URL.
    pub fn get_referrer_url(&self) -> String {
        self.0
            .get_referrer_url
            .and_then(|get_referrer_url| unsafe { get_referrer_url(self.as_ptr()).as_mut() })
            .map(|cef_string| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(cef_string));
                cef_string_userfree_utf16_free(cef_string);
                s
            })
            .unwrap_or_default()
    }
    /// Get the referrer policy.
    pub fn get_referrer_policy(&self) -> ReferrerPolicy {
        self.0
            .get_referrer_policy
            .map(|get_referrer_policy| unsafe {
                ReferrerPolicy::from_unchecked(get_referrer_policy(self.0.as_ptr()) as crate::CEnumType)
            })
            .unwrap_or(ReferrerPolicy::Default)
    }
    /// Get the post data.
    pub fn get_post_data(&self) -> PostData {
        let get_post_data = self.0.get_post_data.unwrap();
        unsafe { PostData::from_ptr_unchecked(get_post_data(self.0.as_ptr())) }
    }
    /// Set the post data.
    pub fn set_post_data(&self, post_data: PostData) {
        if let Some(set_post_data) = self.0.set_post_data {
            unsafe {
                set_post_data(self.0.as_ptr(), post_data.into_raw());
            }
        }
    }
    /// Get the header values. Will not include the Referer value if any.
    pub fn get_header_map(&self) -> HashMap<String, Vec<String>> {
        if let Some(get_header_map) = self.0.get_header_map {
            let map = MultiMap::new();
            unsafe { get_header_map(self.0.as_ptr(), map.as_ptr()) };
            map.into()
        } else {
            HashMap::new()
        }
    }
    /// Returns the first header value for `name` or None if not found.
    /// Will not return the Referer value if any. Use [Request::get_header_map] instead if
    /// `name` might have multiple values.
    pub fn get_header_by_name(&self, name: &str) -> Option<String> {
        self.0
            .get_header_by_name
            .and_then(|get_header_by_name| unsafe {
                get_header_by_name(self.as_ptr(), CefString::new(name).as_ptr()).as_mut()
            })
            .map(|cef_string| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(cef_string));
                cef_string_userfree_utf16_free(cef_string);
                s
            })
    }
    /// Set the header `name` to `value`. if `overwrite` is true any existing
    /// values will be replaced with the new value. if `overwrite` is false any
    /// existing values will not be overwritten. The Referer value cannot be set
    /// using this function.
    pub fn set_header_by_name(&self, name: &str, value: &str, overwrite: bool) {
        if let Some(set_header_by_name) = self.0.set_header_by_name {
            unsafe {
                set_header_by_name(
                    self.0.as_ptr(),
                    CefString::new(name).as_ptr(),
                    CefString::new(value).as_ptr(),
                    overwrite as i32,
                );
            }
        }
    }
    /// Set all values at one time.
    pub fn set(
        &self,
        url: &str,
        method: &str,
        post_data: PostData,
        header_map: HashMap<String, Vec<String>>,
    ) {
        if let Some(set) = self.0.set {
            let url = CefString::new(url);
            let method = CefString::new(method);
            let header_map = MultiMap::from(&header_map);

            unsafe {
                set(
                    self.0.as_ptr(),
                    url.as_ptr(),
                    method.as_ptr(),
                    post_data.into_raw(),
                    header_map.as_ptr(),
                );
            }
        }
    }
    /// Get the flags used in combination with [URLRequest]. See
    /// [URLRequestFlags] for supported values.
    pub fn get_flags(&self) -> Vec<URLRequestFlags> {
        if let Some(get_flags) = self.0.get_flags {
            URLRequestFlags::from_bitfield(unsafe { get_flags(self.0.as_ptr()) })
        } else {
            Vec::new()
        }
    }
    /// Set the flags used in combination with [URLRequest]. See
    /// [URLRequestFlags] for supported values.
    pub fn set_flags(&self, flags: &[URLRequestFlags]) {
        if let Some(set_flags) = self.0.set_flags {
            unsafe {
                set_flags(self.0.as_ptr(), URLRequestFlags::to_bitfield(flags));
            }
        }
    }
    /// Get the URL to the first party for cookies used in combination with
    /// [URLRequest].
    pub fn get_first_party_for_cookies(&self) -> String {
        self.0
            .get_first_party_for_cookies
            .and_then(|get_first_party_for_cookies| unsafe {
                get_first_party_for_cookies(self.as_ptr()).as_mut()
            })
            .map(|cef_string| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(cef_string));
                cef_string_userfree_utf16_free(cef_string);
                s
            })
            .unwrap_or_default()
    }
    /// Set the URL to the first party for cookies used in combination with
    /// [URLRequest].
    pub fn set_first_party_for_cookies(&self, url: &str) {
        if let Some(set_first_party_for_cookies) = self.0.set_first_party_for_cookies {
            unsafe {
                set_first_party_for_cookies(self.0.as_ptr(), CefString::new(url).as_ptr());
            }
        }
    }
    /// Get the resource type for this request. Only available in the browser
    /// process.
    pub fn get_resource_type(&self) -> ResourceType {
        unsafe {
            ResourceType::from_unchecked(((*self.0.as_ptr()).get_resource_type).unwrap()(
                self.0.as_ptr(),
            ) as crate::CEnumType)
        }
    }
    /// Get the transition type for this request. Only available in the browser
    /// process and only applies to requests that represent a main frame or sub-
    /// frame navigation.
    pub fn get_transition_type(&self) -> TransitionType {
        TransitionType::try_from(unsafe {
            (*self.0.as_ptr()).get_transition_type.unwrap()(self.0.as_ptr()).0
        })
        .unwrap()
    }
    /// Returns the globally unique identifier for this request or 0 if not
    /// specified. Can be used by [ResourceRequestHandlerCallbacks] implementations in
    /// the browser process to track a single request across multiple callbacks.
    pub fn get_identifier(&self) -> u64 {
        if let Some(get_identifier) = self.0.get_identifier {
            unsafe { get_identifier(self.0.as_ptr()) }
        } else {
            0
        }
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new()
    }
}

ref_counted_ptr! {
    /// Structure used to represent post data for a web request. The functions of
    /// this structure may be called on any thread.
    pub struct PostData(*mut cef_post_data_t);
}

impl PostData {
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_post_data_create()) }
    }

    /// Returns true if this object is read-only.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.as_ptr()) != 0 })
            .unwrap_or(true)
    }
    /// Returns true if the underlying POST data includes elements that are not
    /// represented by this [PostData] object (for example, multi-part file
    /// upload data). Modifying [PostData] objects with excluded elements may
    /// result in the request failing.
    pub fn has_excluded_elements(&self) -> bool {
        self.0
            .has_excluded_elements
            .map(|has_excluded_elements| unsafe { has_excluded_elements(self.as_ptr()) != 0 })
            .unwrap_or(false)
    }
    /// Returns the number of existing post data elements.
    pub fn get_element_count(&self) -> usize {
        self.0
            .get_element_count
            .map(|get_element_count| unsafe { get_element_count(self.as_ptr()) })
            .unwrap_or(0)
    }
    /// Retrieve the post data elements.
    pub fn get_elements(&self) -> Vec<PostDataElement> {
        let mut count = self.get_element_count();
        if count > 0 {
            if let Some(get_elements) = self.0.get_elements {
                let mut elements = vec![null_mut(); count];
                unsafe {
                    get_elements(self.as_ptr(), &mut count, elements.as_mut_ptr());
                }
                elements
                    .into_iter()
                    .map(|p| unsafe { PostDataElement::from_ptr_unchecked(p) })
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
    /// Remove the specified post data element. Returns true if the removal
    /// succeeds.
    pub fn remove_element(&self, element: &PostDataElement) -> bool {
        if let Some(remove_element) = self.0.remove_element {
            unsafe { remove_element(self.as_ptr(), element.as_ptr()) != 0 }
        } else {
            false
        }
    }
    /// Add the specified post data element. Returns true if the add succeeds.
    pub fn add_element(&self, element: &PostDataElement) -> bool {
        if let Some(add_element) = self.0.add_element {
            unsafe { add_element(self.as_ptr(), element.as_ptr()) != 0 }
        } else {
            false
        }
    }
    /// Remove all existing post data elements.
    pub fn remove_elements(&self) {
        if let Some(remove_elements) = self.0.remove_elements {
            unsafe {
                remove_elements(self.as_ptr());
            }
        }
    }
}

impl Default for PostData {
    fn default() -> Self {
        Self::new()
    }
}

ref_counted_ptr! {
    /// Structure used to represent a single element in the request post data. The
    /// functions of this structure may be called on any thread.
    pub struct PostDataElement(*mut cef_post_data_element_t);
}

impl PostDataElement {
    /// Create a new [PostDataElement] object.
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_post_data_element_create()) }
    }

    /// Returns true if this object is read-only.
    pub fn is_read_only(&self) -> bool {
        self.0
            .is_read_only
            .map(|is_read_only| unsafe { is_read_only(self.as_ptr()) != 0 })
            .unwrap_or(true)
    }
    /// Remove all contents from the post data element.
    pub fn set_to_empty(&self) {
        if let Some(set_to_empty) = self.0.set_to_empty {
            unsafe {
                set_to_empty(self.as_ptr());
            }
        }
    }
    /// The post data element will represent a file.
    pub fn set_to_file(&self, file_name: &str) {
        if let Some(set_to_file) = self.0.set_to_file {
            unsafe {
                set_to_file(self.0.as_ptr(), CefString::new(file_name).as_ptr());
            }
        }
    }
    /// The post data element will represent bytes.  The bytes passed in will be
    /// copied.
    pub fn set_to_bytes(&self, bytes: &[u8]) {
        if let Some(set_to_bytes) = self.0.set_to_bytes {
            unsafe {
                set_to_bytes(
                    self.0.as_ptr(),
                    bytes.len(),
                    bytes.as_ptr() as *const std::ffi::c_void,
                );
            }
        }
    }
    /// Return the type of this post data element.
    pub fn get_type(&self) -> PostDataElementType {
        if let Some(get_type) = self.0.get_type {
            unsafe { PostDataElementType::from_unchecked(get_type(self.as_ptr()) as crate::CEnumType) }
        } else {
            PostDataElementType::Empty
        }
    }
    /// Return the file name.
    pub fn get_file(&self) -> String {
        self.0
            .get_file
            .and_then(|get_file| unsafe { get_file(self.as_ptr()).as_mut() })
            .map(|cef_string| unsafe {
                let s = String::from(CefString::from_ptr_unchecked(cef_string));
                cef_string_userfree_utf16_free(cef_string);
                s
            })
            .unwrap_or_default()
    }
    /// Return the number of bytes.
    pub fn get_bytes_count(&self) -> usize {
        if let Some(get_bytes_count) = self.0.get_bytes_count {
            unsafe { get_bytes_count(self.as_ptr()) }
        } else {
            0
        }
    }
    /// Return the bytes.
    pub fn get_bytes(&self) -> Vec<u8> {
        let size = self.get_bytes_count();
        if size > 0 {
            if let Some(get_bytes) = self.0.get_bytes {
                let mut buffer = vec![0; size];
                unsafe {
                    get_bytes(
                        self.as_ptr(),
                        size,
                        buffer.as_mut_ptr() as *mut std::ffi::c_void,
                    );
                }
                buffer.to_vec()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    }
}

impl Default for PostDataElement {
    fn default() -> Self {
        Self::new()
    }
}
