//! gRPC protobuf definitions and conversions for nockbox-wallet
//!
//! This crate provides protobuf type definitions compatible with nockchain's
//! gRPC API, along with conversion traits to/from nbx-nockchain-types.

// Generated code requires std features from tonic
// We keep this as a std crate since it's only used in non-WASM contexts

// Include the generated protobuf code
pub mod pb {
    pub mod common {
        pub mod v1 {
            include!(concat!(env!("OUT_DIR"), "/nockchain.common.v1.rs"));
        }
        pub mod v2 {
            include!(concat!(env!("OUT_DIR"), "/nockchain.common.v2.rs"));
        }
    }
    pub mod public {
        pub mod v2 {
            include!(concat!(env!("OUT_DIR"), "/nockchain.public.v2.rs"));
        }
    }

    pub const FILE_DESCRIPTOR_SET: &[u8] =
        include_bytes!(concat!(env!("OUT_DIR"), "/nockchain_descriptor.bin"));
}

pub mod client;
pub mod common;
pub mod convert;
