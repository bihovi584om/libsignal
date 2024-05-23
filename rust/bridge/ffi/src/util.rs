//
// Copyright 2020-2021 Signal Messenger, LLC.
// SPDX-License-Identifier: AGPL-3.0-only
//

use attest::enclave::Error as EnclaveError;
use attest::hsm_enclave::Error as HsmEnclaveError;
use device_transfer::Error as DeviceTransferError;
use libsignal_bridge::ffi::*;
use libsignal_net::svr3::Error as Svr3Error;
use libsignal_protocol::*;
use signal_crypto::Error as SignalCryptoError;
use signal_pin::Error as PinError;
use usernames::{UsernameError, UsernameLinkError};
use zkgroup::ZkGroupVerificationFailure;

#[derive(Debug)]
#[repr(C)]
pub enum SignalErrorCode {
    #[allow(dead_code)]
    UnknownError = 1,
    InvalidState = 2,
    InternalError = 3,
    NullParameter = 4,
    InvalidArgument = 5,
    InvalidType = 6,
    InvalidUtf8String = 7,
    Cancelled = 8,

    ProtobufError = 10,

    LegacyCiphertextVersion = 21,
    UnknownCiphertextVersion = 22,
    UnrecognizedMessageVersion = 23,

    InvalidMessage = 30,
    SealedSenderSelfSend = 31,

    InvalidKey = 40,
    InvalidSignature = 41,
    InvalidAttestationData = 42,

    FingerprintVersionMismatch = 51,
    FingerprintParsingError = 52,

    UntrustedIdentity = 60,

    InvalidKeyIdentifier = 70,

    SessionNotFound = 80,
    InvalidRegistrationId = 81,
    InvalidSession = 82,
    InvalidSenderKeySession = 83,

    DuplicatedMessage = 90,

    CallbackError = 100,

    VerificationFailure = 110,

    UsernameCannotBeEmpty = 120,
    UsernameCannotStartWithDigit = 121,
    UsernameMissingSeparator = 122,
    UsernameBadDiscriminatorCharacter = 123,
    UsernameBadNicknameCharacter = 124,
    UsernameTooShort = 125,
    UsernameTooLong = 126,
    UsernameLinkInvalidEntropyDataLength = 127,
    UsernameLinkInvalid = 128,

    UsernameDiscriminatorCannotBeEmpty = 140,
    UsernameDiscriminatorCannotBeZero = 141,
    UsernameDiscriminatorCannotBeSingleDigit = 142,
    UsernameDiscriminatorCannotHaveLeadingZeros = 143,
    UsernameDiscriminatorTooLarge = 144,

    IoError = 130,
    #[allow(dead_code)]
    InvalidMediaInput = 131,
    #[allow(dead_code)]
    UnsupportedMediaInput = 132,

    ConnectionTimedOut = 133,
    NetworkProtocol = 134,
    RateLimited = 135,
    WebSocket = 136,
    CdsiInvalidToken = 137,
    ConnectionFailed = 138,
    ChatServiceInactive = 139,

    SvrDataMissing = 150,
    SvrRestoreFailed = 151,

    AppExpired = 160,
    DeviceDeregistered = 161,
}

impl From<&SignalFfiError> for SignalErrorCode {
    fn from(err: &SignalFfiError) -> Self {
        match err {
            SignalFfiError::NullPointer => SignalErrorCode::NullParameter,

            SignalFfiError::UnexpectedPanic(_)
            | SignalFfiError::InternalError(_)
            | SignalFfiError::DeviceTransfer(DeviceTransferError::InternalError(_))
            | SignalFfiError::Signal(SignalProtocolError::FfiBindingError(_)) => {
                SignalErrorCode::InternalError
            }

            SignalFfiError::InvalidUtf8String => SignalErrorCode::InvalidUtf8String,

            SignalFfiError::Cancelled => SignalErrorCode::Cancelled,

            SignalFfiError::Signal(SignalProtocolError::InvalidProtobufEncoding) => {
                SignalErrorCode::ProtobufError
            }

            SignalFfiError::Signal(SignalProtocolError::DuplicatedMessage(_, _)) => {
                SignalErrorCode::DuplicatedMessage
            }

            SignalFfiError::Signal(SignalProtocolError::InvalidPreKeyId)
            | SignalFfiError::Signal(SignalProtocolError::InvalidSignedPreKeyId)
            | SignalFfiError::Signal(SignalProtocolError::InvalidKyberPreKeyId) => {
                SignalErrorCode::InvalidKeyIdentifier
            }

            SignalFfiError::Signal(SignalProtocolError::SealedSenderSelfSend) => {
                SignalErrorCode::SealedSenderSelfSend
            }

            SignalFfiError::Signal(SignalProtocolError::SignatureValidationFailed) => {
                SignalErrorCode::InvalidSignature
            }

            SignalFfiError::Signal(SignalProtocolError::NoKeyTypeIdentifier)
            | SignalFfiError::Signal(SignalProtocolError::BadKeyType(_))
            | SignalFfiError::Signal(SignalProtocolError::BadKeyLength(_, _))
            | SignalFfiError::Signal(SignalProtocolError::BadKEMKeyType(_))
            | SignalFfiError::Signal(SignalProtocolError::WrongKEMKeyType(_, _))
            | SignalFfiError::Signal(SignalProtocolError::BadKEMKeyLength(_, _))
            | SignalFfiError::Signal(SignalProtocolError::InvalidMacKeyLength(_))
            | SignalFfiError::DeviceTransfer(DeviceTransferError::KeyDecodingFailed)
            | SignalFfiError::HsmEnclave(HsmEnclaveError::InvalidPublicKeyError)
            | SignalFfiError::SignalCrypto(SignalCryptoError::InvalidKeySize) => {
                SignalErrorCode::InvalidKey
            }

            SignalFfiError::Sgx(EnclaveError::AttestationDataError { .. }) => {
                SignalErrorCode::InvalidAttestationData
            }

            SignalFfiError::Pin(PinError::Argon2Error(_))
            | SignalFfiError::Pin(PinError::DecodingError(_))
            | SignalFfiError::Pin(PinError::MrenclaveLookupError) => {
                SignalErrorCode::InvalidArgument
            }

            SignalFfiError::Signal(SignalProtocolError::SessionNotFound(_))
            | SignalFfiError::Signal(SignalProtocolError::NoSenderKeyState { .. }) => {
                SignalErrorCode::SessionNotFound
            }

            SignalFfiError::Signal(SignalProtocolError::InvalidRegistrationId(..)) => {
                SignalErrorCode::InvalidRegistrationId
            }

            SignalFfiError::Signal(SignalProtocolError::FingerprintParsingError) => {
                SignalErrorCode::FingerprintParsingError
            }

            SignalFfiError::Signal(SignalProtocolError::FingerprintVersionMismatch(_, _)) => {
                SignalErrorCode::FingerprintVersionMismatch
            }

            SignalFfiError::Signal(SignalProtocolError::UnrecognizedMessageVersion(_))
            | SignalFfiError::Signal(SignalProtocolError::UnknownSealedSenderVersion(_)) => {
                SignalErrorCode::UnrecognizedMessageVersion
            }

            SignalFfiError::Signal(SignalProtocolError::UnrecognizedCiphertextVersion(_)) => {
                SignalErrorCode::UnknownCiphertextVersion
            }

            SignalFfiError::Signal(SignalProtocolError::InvalidMessage(..))
            | SignalFfiError::Signal(SignalProtocolError::CiphertextMessageTooShort(_))
            | SignalFfiError::Signal(SignalProtocolError::InvalidSealedSenderMessage(_))
            | SignalFfiError::Signal(SignalProtocolError::BadKEMCiphertextLength(_, _))
            | SignalFfiError::SignalCrypto(SignalCryptoError::InvalidTag)
            | SignalFfiError::Sgx(EnclaveError::AttestationError(_))
            | SignalFfiError::Sgx(EnclaveError::NoiseError(_))
            | SignalFfiError::Sgx(EnclaveError::NoiseHandshakeError(_))
            | SignalFfiError::HsmEnclave(HsmEnclaveError::HSMHandshakeError(_))
            | SignalFfiError::HsmEnclave(HsmEnclaveError::HSMCommunicationError(_)) => {
                SignalErrorCode::InvalidMessage
            }

            SignalFfiError::Signal(SignalProtocolError::LegacyCiphertextVersion(_)) => {
                SignalErrorCode::LegacyCiphertextVersion
            }

            SignalFfiError::Signal(SignalProtocolError::UntrustedIdentity(_))
            | SignalFfiError::HsmEnclave(HsmEnclaveError::TrustedCodeError) => {
                SignalErrorCode::UntrustedIdentity
            }

            SignalFfiError::Signal(SignalProtocolError::InvalidState(_, _))
            | SignalFfiError::Sgx(EnclaveError::InvalidBridgeStateError)
            | SignalFfiError::HsmEnclave(HsmEnclaveError::InvalidBridgeStateError) => {
                SignalErrorCode::InvalidState
            }

            SignalFfiError::Signal(SignalProtocolError::InvalidSessionStructure(_)) => {
                SignalErrorCode::InvalidSession
            }

            SignalFfiError::Signal(SignalProtocolError::InvalidSenderKeySession { .. }) => {
                SignalErrorCode::InvalidSenderKeySession
            }

            SignalFfiError::InvalidArgument(_)
            | SignalFfiError::Signal(SignalProtocolError::InvalidArgument(_))
            | SignalFfiError::HsmEnclave(HsmEnclaveError::InvalidCodeHashError)
            | SignalFfiError::SignalCrypto(_) => SignalErrorCode::InvalidArgument,

            SignalFfiError::Signal(SignalProtocolError::ApplicationCallbackError(_, _)) => {
                SignalErrorCode::CallbackError
            }

            SignalFfiError::ZkGroupVerificationFailure(ZkGroupVerificationFailure) => {
                SignalErrorCode::VerificationFailure
            }

            SignalFfiError::ZkGroupDeserializationFailure(_) => SignalErrorCode::InvalidType,

            SignalFfiError::UsernameError(UsernameError::NicknameCannotBeEmpty) => {
                SignalErrorCode::UsernameCannotBeEmpty
            }

            SignalFfiError::UsernameError(UsernameError::NicknameCannotStartWithDigit) => {
                SignalErrorCode::UsernameCannotStartWithDigit
            }

            SignalFfiError::UsernameError(UsernameError::MissingSeparator) => {
                SignalErrorCode::UsernameMissingSeparator
            }

            SignalFfiError::UsernameError(UsernameError::BadNicknameCharacter) => {
                SignalErrorCode::UsernameBadNicknameCharacter
            }

            SignalFfiError::UsernameError(UsernameError::NicknameTooShort) => {
                SignalErrorCode::UsernameTooShort
            }

            SignalFfiError::UsernameError(UsernameError::NicknameTooLong)
            | SignalFfiError::UsernameLinkError(UsernameLinkError::InputDataTooLong) => {
                SignalErrorCode::UsernameTooLong
            }

            SignalFfiError::UsernameError(UsernameError::DiscriminatorCannotBeEmpty) => {
                SignalErrorCode::UsernameDiscriminatorCannotBeEmpty
            }

            SignalFfiError::UsernameError(UsernameError::DiscriminatorCannotBeZero) => {
                SignalErrorCode::UsernameDiscriminatorCannotBeZero
            }

            SignalFfiError::UsernameError(UsernameError::DiscriminatorCannotBeSingleDigit) => {
                SignalErrorCode::UsernameDiscriminatorCannotBeSingleDigit
            }

            SignalFfiError::UsernameError(UsernameError::DiscriminatorCannotHaveLeadingZeros) => {
                SignalErrorCode::UsernameDiscriminatorCannotHaveLeadingZeros
            }

            SignalFfiError::UsernameError(UsernameError::BadDiscriminatorCharacter) => {
                SignalErrorCode::UsernameBadDiscriminatorCharacter
            }

            SignalFfiError::UsernameError(UsernameError::DiscriminatorTooLarge) => {
                SignalErrorCode::UsernameDiscriminatorTooLarge
            }

            SignalFfiError::UsernameProofError(usernames::ProofVerificationFailure) => {
                SignalErrorCode::VerificationFailure
            }

            SignalFfiError::UsernameLinkError(UsernameLinkError::InvalidEntropyDataLength) => {
                SignalErrorCode::UsernameLinkInvalidEntropyDataLength
            }

            SignalFfiError::UsernameLinkError(_) => SignalErrorCode::UsernameLinkInvalid,

            SignalFfiError::Io(_) => SignalErrorCode::IoError,

            #[cfg(feature = "signal-media")]
            SignalFfiError::Mp4SanitizeParse(err) => {
                use signal_media::sanitize::mp4::ParseError;
                match err.kind {
                    ParseError::InvalidBoxLayout { .. }
                    | ParseError::InvalidInput { .. }
                    | ParseError::MissingRequiredBox { .. }
                    | ParseError::TruncatedBox => SignalErrorCode::InvalidMediaInput,

                    ParseError::UnsupportedBoxLayout { .. }
                    | ParseError::UnsupportedBox { .. }
                    | ParseError::UnsupportedFormat { .. } => {
                        SignalErrorCode::UnsupportedMediaInput
                    }
                }
            }

            #[cfg(feature = "signal-media")]
            SignalFfiError::WebpSanitizeParse(err) => {
                use signal_media::sanitize::webp::ParseError;
                match err.kind {
                    ParseError::InvalidChunkLayout { .. }
                    | ParseError::InvalidInput { .. }
                    | ParseError::InvalidVp8lPrefixCode { .. }
                    | ParseError::MissingRequiredChunk { .. }
                    | ParseError::TruncatedChunk => SignalErrorCode::InvalidMediaInput,

                    ParseError::UnsupportedChunk { .. }
                    | ParseError::UnsupportedVp8lVersion { .. } => {
                        SignalErrorCode::UnsupportedMediaInput
                    }
                }
            }
            SignalFfiError::WebSocket(_) => SignalErrorCode::WebSocket,
            SignalFfiError::ConnectionTimedOut => SignalErrorCode::ConnectionTimedOut,
            SignalFfiError::ConnectionFailed => SignalErrorCode::ConnectionFailed,
            SignalFfiError::ChatServiceInactive => SignalErrorCode::ChatServiceInactive,
            SignalFfiError::AppExpired => SignalErrorCode::AppExpired,
            SignalFfiError::DeviceDeregistered => SignalErrorCode::DeviceDeregistered,
            SignalFfiError::NetworkProtocol(_) => SignalErrorCode::NetworkProtocol,
            SignalFfiError::CdsiInvalidToken => SignalErrorCode::CdsiInvalidToken,
            SignalFfiError::RateLimited {
                retry_after_seconds: _,
            } => SignalErrorCode::RateLimited,
            SignalFfiError::Svr(Svr3Error::DataMissing) => SignalErrorCode::SvrDataMissing,
            SignalFfiError::Svr(Svr3Error::RestoreFailed(_)) => SignalErrorCode::SvrRestoreFailed,
            SignalFfiError::Svr(_) => SignalErrorCode::UnknownError,
        }
    }
}
