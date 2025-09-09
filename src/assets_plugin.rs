use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>();
        app
            .add_loading_state(
                LoadingState::new(GameState::AssetLoading)
                    .continue_to_state(GameState::InGame)
                    .load_collection::<PlayerAssets>(),
            );
    }
}

#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(texture_atlas_layout(tile_size_x = 500, tile_size_y = 500, columns = 1, rows = 2, padding_x = 0, padding_y = 0, offset_x = 0, offset_y = 0))]
    pub layout: Handle<TextureAtlasLayout>,
    #[asset(path = "player.png")]
    pub sprite: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
pub enum GameState {
    #[default]
    AssetLoading,
    InGame,
}
