fn main() {
    #[cfg(feature = "sqlite")]
    migration_build::generate();
}
