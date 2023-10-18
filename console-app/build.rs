fn main() {
    cynic_codegen::register_schema("tibber")
        .from_sdl_file("tibber.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
