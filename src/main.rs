//use wasm_component_layer::*;
// /*
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxBuilder, ResourceTable};
use wasmtime::component::{Component, Linker, bindgen};
//*/
use wac_graph::{types::Package, CompositionGraph, EncodeOptions};
use wasmtime_wasi::preview1::WasiP1Ctx;

struct MyState {
    table: ResourceTable,
    ctx: WasiCtx,
}

impl WasiView for MyState {
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }

    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
}

impl MyState {
    fn new() -> MyState {
        let mut wasi = WasiCtxBuilder::new();
        wasi.arg("../composition.wasm");
        wasi.arg("--help");

        MyState {
            ctx: wasi.build(),
            table: ResourceTable::new(),
        }
    }
}

fn main() {
    /** Composition step
     * This step generates the composed component from the two wasm components in the assets folder following the specification given by the wit
     */
    let mut graph = CompositionGraph::new();
    // Register the package dependencies into the graph
    let package = Package::from_file(
        "rpn",
        None,
        "./assets/rpn.wasm",
        graph.types_mut(),
    )
        .unwrap();
    let rpn = graph.register_package(package).unwrap();
    let package = Package::from_file(
        "command",
        None,
        "./assets/command.wasm",
        graph.types_mut(),
    )
        .unwrap();


    let command = graph.register_package(package).unwrap();

    let rpn_instance = graph.instantiate(rpn);
    let command_instance = graph.instantiate(command);

    let rpn_export = graph
        .alias_instance_export(rpn_instance, "component:rpn/types@0.1.0")
        .unwrap();

    graph
        .set_instantiation_argument(command_instance, "component:rpn/types@0.1.0", rpn_export)
        .unwrap();

    let command_export = graph
        .alias_instance_export(command_instance, "wasi:cli/run@0.2.0")
        .unwrap();

    graph.export(command_export, "wasi:cli/run@0.2.0").unwrap();

    // Encode the graph into a WASM binary
    let encoding = graph.encode(EncodeOptions::default()).unwrap();
    std::fs::write("composition.wasm", encoding).unwrap();

    /** Execution step
     * Here I am trying (unsuccessfully :[ ) to use wasmtime programmatically to run the composed component
    */
    const WASM: &[u8] = include_bytes!("../composition.wasm");

    let mut config = Config::new();
    //config.async_support(true);
    config.wasm_component_model(true);

    let engine = Engine::new(&config).unwrap();
    let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
    //let mut linker = Linker::<MyState>::new(&engine);


    let mut builder = WasiCtxBuilder::new();
    let mut store = Store::new(
        &engine,
        builder.build_p1(),
    );


    let component = Component::new(&engine, WASM).unwrap();
    let instance = linker.instantiate(&mut store, &component).unwrap();
    let func = instance.get_func(&mut store, "root:wasi:cli/run").unwrap();
    func.call(&mut store, &[], &mut []).unwrap();


    // These lines only work if you generate a wit from the command.wasm binary and
    // add: bindgen!("root" in "./assets/command.wit");

    //let root = Root::instantiate(&mut store, &component, &mut linker).unwrap();
    //let run = root.wasi_cli_run();
    //run.call_run(&mut store).unwrap().expect("Couldn't call run...");
    //let root = Root::instantiate(&mut store, &component, &linker).unwrap();
    //root.wasi_cli_run().call_run(&mut store).unwrap().expect("Something went wrong, couldn't call run");




    // WASM COMPONENT LAYER implementation is worthless because the tool does not support composed components see
    // https://github.com/DouglasDwyer/wasm_component_layer/issues/21
    /*
    let engine = wasm_component_layer::Engine::new(wasmtime_runtime_layer::Engine::default());
    let mut store = wasm_component_layer::Store::new(&engine, WASM);

    let composition = wasm_component_layer::Component::new(&engine, WASM).unwrap();
    let linker = wasm_component_layer::Linker::default();
    let instance = linker.instantiate(&mut store, &composition).unwrap();

    let interface = instance
        .exports()
        .instance(&"wasi:cli/run".try_into().unwrap())
        .unwrap();

    let run = interface
        .func("run")
        .unwrap()
        .typed::<(), String>()
        .unwrap();

    println!("Calling `run` function: {}",
        run.call(&mut store, ()).unwrap()
    );
    */
}