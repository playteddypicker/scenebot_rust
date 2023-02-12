use crate::utils::scene_core::ImageSize;
use serenity::model::id::GuildId;
use std::num::NonZeroU64;

pub struct GuildConfig {
    pub guild_id: NonZeroU64,
    pub auto_magnitute_enable: bool,
    pub auto_magnitute_config: ImageSize,
}

impl GuildConfig {
    fn new(&self, guild: &GuildId) -> Self {
        Self {
            guild_id: guild.0,
            auto_magnitute_enable: false,
            auto_magnitute_config: ImageSize::Auto,
        }
    }
}
