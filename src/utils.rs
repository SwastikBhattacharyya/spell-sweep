use std::error::Error;
use std::hash::{DefaultHasher, Hash, Hasher};

use rkyv::{
    de::deserializers::{SharedDeserializeMap, SharedDeserializeMapError},
    ser::serializers::{
        AlignedSerializer, AllocScratch, AllocScratchError, CompositeSerializer,
        CompositeSerializerError, FallbackScratch, HeapScratch, SharedSerializeMap,
        SharedSerializeMapError,
    },
    validation::{
        validators::{CheckDeserializeError, DefaultValidator, DefaultValidatorError},
        CheckArchiveError,
    },
    AlignedVec, Archive, Deserialize, Serialize,
};

pub fn hash_with_seed(input: &str, seed: u32) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write_u32(seed);
    input.hash(&mut hasher);

    return hasher.finish();
}

pub fn serialize<'a, T>(
    value: &T,
) -> Result<
    AlignedVec,
    CompositeSerializerError<std::convert::Infallible, AllocScratchError, SharedSerializeMapError>,
>
where
    T: Serialize<
        CompositeSerializer<
            AlignedSerializer<AlignedVec>,
            FallbackScratch<HeapScratch<256>, AllocScratch>,
            SharedSerializeMap,
        >,
    >,
{
    rkyv::to_bytes::<T, 256>(value)
}

pub fn deserialize<'a, T: Archive>(
    bytes: &'a [u8],
) -> Result<
    T,
    CheckDeserializeError<
        CheckArchiveError<
            <<T as Archive>::Archived as rkyv::CheckBytes<DefaultValidator<'_>>>::Error,
            DefaultValidatorError,
        >,
        SharedDeserializeMapError,
    >,
>
where
    <T as Archive>::Archived: rkyv::CheckBytes<DefaultValidator<'a>>,
    <T as Archive>::Archived: Deserialize<T, SharedDeserializeMap>,
    <T as Archive>::Archived: 'a,
{
    rkyv::from_bytes::<T>(bytes)
}
