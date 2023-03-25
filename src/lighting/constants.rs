use bevy::{prelude::{HandleUntyped, Shader}, reflect::TypeUuid};

pub(super) const SCREEN_PROBE_SIZE: i32 = 16;

pub(super) const SHADER_CAMERA: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1371231089456109822);
pub(super) const SHADER_TYPES: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 4462033275253590181);
pub(super) const SHADER_ATTENUATION: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 5254739165481917368);
pub(super) const SHADER_HALTON: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1287391288877821366);
pub(super) const SHADER_MATH: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 2387462894328787238);