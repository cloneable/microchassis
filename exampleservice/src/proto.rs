#![allow(clippy::derive_partial_eq_without_eq)]

tonic::include_proto!("exampleservice.v1");

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("exampleservice_descriptor");
