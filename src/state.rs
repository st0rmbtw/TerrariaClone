
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    AssetLoading,
    MainMenu,
    WorldLoading,
    InGame,
    Paused
}