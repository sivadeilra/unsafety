//! Provides annotations for describing and auditing usages of `unsafe` code in Rust.
//!
//! This crate has no effect on the compilation or runtime behavior of Rust code. Its
//! purpose is to allow developers to annotate Rust code with information about _why_
//! unsafe code is used, and to enable automated tools for auditing code bases that
//! use unsafe code.
//!
//! Instead of this:
//!
//! ```no_run
//! # mod libc { #[allow(non_camel_case_types)] pub type c_void = u8; }
//! # fn allocate_foreign_object() -> *mut libc::c_void { unimplemented!(); }
//! # fn use_foreign_object(_: *mut libc::c_void, _: u8) { unimplemented!(); }
//! # fn free_foreign_object(_: *mut libc::c_void) { unimplemented!(); }
//! unsafe {
//!     // Scary interop code:
//!     let ptr: *mut libc::c_void = allocate_foreign_object();
//!     use_foreign_object(ptr, 42);
//!     free_foreign_object(ptr);
//! }
//! ```
//!
//! Developers can do this:
//!
//! ```no_run
//! # mod libc { #[allow(non_camel_case_types)] pub type c_void = u8; }
//! # fn allocate_foreign_object() -> *mut libc::c_void { unimplemented!(); }
//! # fn use_foreign_object(_: *mut libc::c_void, _: u8) { unimplemented!(); }
//! # fn free_foreign_object(_: *mut libc::c_void) { unimplemented!(); }
//! use unsafety::{unsafe_because, USES_FOREIGN_CODE};
//!
//! unsafe_because! {
//!     USES_FOREIGN_CODE => {
//!         // Scary interop code:
//!         let ptr: *mut libc::c_void = allocate_foreign_object();
//!         use_foreign_object(ptr, 42);
//!         free_foreign_object(ptr);
//!     }
//! }
//! ```
//!
//! Type safety and concurrency safety are the key benefits of Rust. Because
//! these safety properties depend on _all components_ in that system correctly
//! respecting those properties, even unsafe code, it is crucial that `unsafe`
//! code nevertheless be _correct_ code. This crate is intended to help meet
//! that goal, by allowing developers to describe _why_ code does what it does,
//! with respect to unsafe code, and to make it easy to audit those
//! descriptions.
//!
//! # Annotating reasons
//!
//! The `unsafe_because` macro requires you to give a reason, and it allows you
//! to give additional, optional information. You can add the following to
//! any invocation of `unsafe_because`. (All of these can be repeated.)
//!
//! * `reason.owner("foo")`: Identifies an owner or expert in this part of the
//!   design.
//! * `reason.bug("...")`: An identifier in a bug-tracking system. This is
//!   typically a URL or a bug number.
//! * `reason.link("http://...")`: A link to any relevant web page, such as
//!   a design document.
//! * `reason.tag("key", "value")`: Allows you to specify arbitrary key-value pairs.
//!
//! # Reusing reasons
//!
//! Instead of re-stating the same reason repeatedly, reasons can be defined as
//! constants and reused. This is useful when a reason has annotations, which
//! would be cumbersome to repeat at every usage. Example:
//!
//! ```no_run
//! use unsafety::{UnsafeReason, IMPLEMENTS_DEVICE_DRIVER, unsafe_because};
//!
//! const IMPLEMENTS_FANCY_NETWORK_DRIVER: UnsafeReason = IMPLEMENTS_DEVICE_DRIVER
//!     .bug("some_bug_link")
//!     .owner("foo")
//!     .owner("bar")
//!     .link("https://.../some_design_doc.html");
//!
//! unsafe_because! {
//!     IMPLEMENTS_FANCY_NETWORK_DRIVER => {
//!         // ...
//!     }
//! }
//!
//! unsafe_because! {
//!     IMPLEMENTS_FANCY_NETWORK_DRIVER => {
//!         // ... even more code ...
//!     }
//! }
//! ```
//!
//! # Combining reasons
//!
//! Sometimes a single `unsafe` block has more than reason for using `unsafe` code.
//! If possible, developers should split such blocks into separate blocks and use
//! separate justifications for them. However, at times that is not possible.
//! `unsafe_because!` allows you to provide a list of reasons, within square brackets.
//!
//! Example:
//!
//! ```no_run
//! use unsafety::{PERFORMANCE, IMPLEMENTS_DEVICE_DRIVER, unsafe_because};
//!
//! // Some code has more than one reason for requiring unsafe code.
//! unsafe_because! {
//!     [PERFORMANCE, IMPLEMENTS_DEVICE_DRIVER] =>
//!         println!("Super fast and scary (but correct) code goes here.");
//! }
//! ```
//!
//! # TODO
//!
//! * Improve the list of standard reasons.
//! * Auditing tools.
//! * Needs macros for defining unsafe traits and unsafe function signatures, not
//!   just unsafe code blocks.
//!
//! # Future direction
//!
//! It is possible that some future version of Rust could verify that a particular
//! _set_ of usages of `unsafe` meet some requirement. For example, it might be
//! useful to allow unsafe code for the reason of accessing a device driver, but
//! no other reason, within a given crate. `unsafe_because` could allow developers
//! to encode that knowledge now, rather than trying to re-discover that knowledge
//! after a large, mature component has been developed.
//!

#![no_std]

/// Represents an annotation on an unsafe code block or item. Because these annotations
/// are intended to have no effect on code generation, this type is empty.
pub struct UnsafeReason {}

impl UnsafeReason {
    /// Starts a new annotation block, given a reason identifier.
    pub const fn new(_reason_id: &'static str) -> Self {
        Self {}
    }

    /// An annotation which identifies a bug. This might be a simple identifier, such as `42`,
    /// although it will typically be a URL in a bug tracking database.
    pub const fn bug(self, _bug_id: &'static str) -> Self {
        self
    }

    /// An annotation which is an arbitrary message to the reader. This is different from
    /// simple code comments because this annotation will be noticed by auditing tools.
    pub const fn message(self, _message: &'static str) -> Self {
        self
    }

    /// An annotation which is the name, user id, or email address of an owner or otherwise
    /// accountable person.
    pub const fn owner(self, _owner: &'static str) -> Self {
        self
    }

    /// An annotation which is a link (URL) to a relevant document, such as a design document.
    pub const fn link(self, _url: &'static str) -> Self {
        self
    }

    /// An annotation which is an arbitrary key-value pair.
    pub const fn tag(self, _tag: &'static str, _value: &'static str) -> Self {
        self
    }
}

/// Annotations a block of unsafe code. See module docs.
/// 
/// This macro uses `reason => body` syntax in order to avoid the "right-ward creep"
/// that would occur if the body was always wrapped in another level of braces.
#[macro_export]
macro_rules! unsafe_because {
    (
        [
            $(
                $reason:expr
            ),+
        ] => $($body:tt)*
    ) => {
        {
            $(
                $crate::unsafe_reason($reason);
            )*
            unsafe {
                $($body)*
            }
        }
    };
    (
        $reason:expr => $($body:tt)*
    ) => {
        {
            $crate::unsafe_reason($reason);
            unsafe {
                $($body)*
            }
        }
    }
}

/// This function does nothing. It exists only so that the `unsafe_because` macro can
/// verify that the reasons given to it are syntactically valid.
#[inline(always)]
pub const fn unsafe_reason(_reason: UnsafeReason) {
    // nothing
}

macro_rules! standard_reasons {
    ( $(
        $(#[$a:meta])*
        $name:ident,
    )* ) => {
        $(
            $(#[$a])*
            pub const $name: $crate::UnsafeReason = $crate::UnsafeReason::new(stringify!($name));
        )*
    }
}

standard_reasons! {
    /// The unsafe code calls foreign code (such as C code). Such code cannot be verified
    /// by Rust's safety rules, and hence is unsafe.
    USES_FOREIGN_CODE,

    /// The unsafe code is called by foreign code (such as C code). The unsafe code is
    /// necessary in order to correctly exchange data and control flow with the calling
    /// code.
    USED_BY_FOREIGN_CODE,

    /// The unsafe code safely implements an algorithm that requires maximum performance.
    /// It is responsible for ensuring bounds checks, overflow checks, etc. have been
    /// performed.
    PERFORMANCE,

    /// The unsafe code safely implements a legal type conversion that cannot currently
    /// be expressed in Rust's type system.
    IMPLEMENTS_SAFE_TRANSMUTE,

    /// Implements a container type, such as `Vec`, `HashMap`, etc.
    IMPLEMENTS_CONTAINER,

    /// The unsafe code is part of a device driver implementation. It must be able to
    /// directly access memory. For example, it needs to be able to directly access
    /// memory-mapped I/O registers (MMIO).
    IMPLEMENTS_DEVICE_DRIVER,

    /// The unsafe code is part of the implementation of a memory manager, such as a
    /// heap or a page table. This is distinct from `ImplementsContainer` because a
    /// container implementation uses a memory manager, but is not part of the
    /// implementation of a memory manager.
    IMPLEMENTS_MEMORY_MANAGER,

    /// The unsafe code uses processor-specific intrinsics, such as vector (SIMD)
    /// intrinsics. Some of these intrinsics are marked `unsafe` because they are
    /// not guaranteed to be present in all processors. (For example, SSE 4.)
    /// Using an intrinsic instruction on a processor that does not implement the
    /// intrinsic is undefined behavior.
    USES_VECTOR_INTRINSICS,
}
