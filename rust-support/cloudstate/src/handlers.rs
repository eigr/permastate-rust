pub mod handler {

    pub struct CommandContext;

    pub trait CommandHandler<T> {
        fn get() -> T;
        fn handle_command(entity: T, ctx: CommandContext) -> T;
    }

    pub trait EventHandler<T> {
        fn handle_event(entity: T);
    }

    pub trait SnapshotHandler<T> {
        fn snapshot() -> T;
        fn handle_snapshot(entity: T);
    }
}