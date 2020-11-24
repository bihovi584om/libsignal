//
// Copyright 2020 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

use jni::JNIEnv;
use jni::objects::JString;
use jni::sys::{JNI_FALSE, JNI_TRUE};
use libsignal_protocol_rust::*;
use std::borrow::Borrow;
use std::ops::Deref;

use crate::jni::*;

pub(crate) trait ArgTypeInfo<'a>: Sized {
    type ArgType;
    fn convert_from(env: &JNIEnv<'a>, foreign: Self::ArgType) -> Result<Self, SignalJniError>;
}

pub(crate) trait RefArgTypeInfo<'a>: Deref {
    type ArgType;
    type StoredType: Borrow<Self::Target> + 'a;
    fn convert_from(env: &JNIEnv<'a>, foreign: Self::ArgType) -> Result<Self::StoredType, SignalJniError>;
}

pub(crate) trait ResultTypeInfo<'a>: Sized {
    type ResultType;
    fn convert_into(self, env: &JNIEnv<'a>) -> Result<Self::ResultType, SignalJniError>;
}

impl<'a> ArgTypeInfo<'a> for u32 {
    type ArgType = jint;
    fn convert_from(_env: &JNIEnv<'a>, foreign: jint) -> Result<Self, SignalJniError> {
        jint_to_u32(foreign)
    }
}

impl<'a> ArgTypeInfo<'a> for String {
    type ArgType = JString<'a>;
    fn convert_from(env: &JNIEnv<'a>, foreign: JString<'a>) -> Result<Self, SignalJniError> {
        Ok(env.get_string(foreign)?.into())
    }
}

impl<'a> RefArgTypeInfo<'a> for &'_ [u8] {
    type ArgType = jbyteArray;
    type StoredType = Vec<u8>;
    fn convert_from(env: &JNIEnv<'a>, foreign: Self::ArgType) -> Result<Vec<u8>, SignalJniError> {
        Ok(env.convert_byte_array(foreign)?)
    }
}

impl<'a> ResultTypeInfo<'a> for bool {
    type ResultType = jboolean;
    fn convert_into(self, _env: &JNIEnv<'a>) -> Result<Self::ResultType, SignalJniError> {
        Ok(if self { JNI_TRUE } else { JNI_FALSE })
    }
}

impl<'a, T: ResultTypeInfo<'a>> ResultTypeInfo<'a> for Result<T, SignalProtocolError> {
    type ResultType = T::ResultType;
    fn convert_into(self, env: &JNIEnv<'a>) -> Result<Self::ResultType, SignalJniError> {
        T::convert_into(self?, env)
    }
}

macro_rules! native_handle {
    ($typ:ty) => {
        impl<'a> RefArgTypeInfo<'a> for &$typ {
            type ArgType = ObjectHandle;
            type StoredType = &'static $typ;
            fn convert_from(_env: &JNIEnv<'a>, foreign: Self::ArgType) -> Result<Self::StoredType, SignalJniError> {
                Ok(unsafe { native_handle_cast(foreign) }?)
            }
        }
        impl<'a> ResultTypeInfo<'a> for $typ {
            type ResultType = ObjectHandle;
            fn convert_into(self, _env: &JNIEnv<'a>) -> Result<Self::ResultType, SignalJniError> {
                box_object(Ok(self))
            }
        }
    }
}

native_handle!(PublicKey);
native_handle!(ProtocolAddress);

macro_rules! trivial {
    ($typ:ty) => {
        impl<'a> ArgTypeInfo<'a> for $typ {
            type ArgType = Self;
            fn convert_from(_env: &JNIEnv<'a>, foreign: Self) -> Result<Self, SignalJniError> { Ok(foreign) }
        }
        impl<'a> ResultTypeInfo<'a> for $typ {
            type ResultType = Self;
            fn convert_into(self, _env: &JNIEnv<'a>) -> Result<Self, SignalJniError> { Ok(self) }
        }
    }
}

trivial!(i32);

macro_rules! jni_arg_type {
    (u32) => (jni::jint);
    (String) => (jni::JString);
    (&[u8]) => (jni::jbyteArray);
    (& $typ:ty) => (jni::ObjectHandle);
}

macro_rules! jni_result_type {
    (Result<$typ:tt, $_:tt>) => (jni_result_type!($typ));
    (bool) => (jni::jboolean);
    (i32) => (jni::jint);
    ( $typ:ty ) => (jni::ObjectHandle);
}
