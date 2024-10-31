use mlua::FromLua;
use tealr::{
    mlu::{
        mlua::{Lua, Result},
        TealData, TealDataMethods, UserData, UserDataProxy,
    },
    ToTypename, TypeWalker,
};
//this example shows how to expose a `proxy` type to enable calling static global functions from anywhere.

//First, create the struct you want to export to lua.
//instead of both deriving UserData and TypeName you can also
//derive TealDerive, which does both. However you will still need to import
//UserData and TypeName
//The clone is only needed because one of the example functions has it as a parameter
#[derive(Clone, UserData, ToTypename)]
struct Example {
    float: f32,
}

impl<'lua> FromLua<'lua> for Example {
    fn from_lua(value: mlua::prelude::LuaValue<'lua>, _: &'lua Lua) -> Result<Self> {
        value
            .as_userdata()
            .map(|x| x.take())
            .unwrap_or(Err(mlua::Error::FromLuaConversionError {
                from: value.type_name(),
                to: "Example",
                message: None,
            }))
    }
}

//now, implement TealData. This tells rlua what methods are available and tealr what the types are
impl TealData for Example {
    //implement your methods/functions
    fn add_methods<'lua, T: TealDataMethods<'lua, Self>>(methods: &mut T) {
        methods.add_method("example_method", |_, _, x: i8| Ok(x));
        methods.add_method_mut("example_method_mut", |_, _, x: (i8, String)| Ok(x.1));
        methods.add_function("example_function", |_, x: Vec<String>| Ok((x, 8)));
        methods.add_function_mut("example_function_mut", |_, x: (bool, Option<Example>)| {
            Ok(x)
        })
    }

    fn add_fields<'lua, F: tealr::mlu::TealDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("example_field", |_, s: &Example| Ok(s.float));
        fields.add_field_method_set("example_field_set", |_, s: &mut Example, v: f32| {
            s.float = v;
            Ok(())
        });
        fields.add_field_function_get("example_static_field", |_, _| Ok("my_field"));
        fields.add_field_function_get("example_static_field_mut", |_, _| Ok("my_mut_field"));
    }
}

// document and expose the global proxy
#[derive(Default)]
struct Export;
impl tealr::mlu::ExportInstances for Export {
    fn add_instances<'lua, T: tealr::mlu::InstanceCollector<'lua>>(
        self,
        instance_collector: &mut T,
    ) -> mlua::Result<()> {
        instance_collector.document_instance("Documentation for the exposed static proxy");

        // note that the proxy type is NOT `Example` but a special mlua type, which is represented differnetly in .d.tl as well
        instance_collector.add_instance("Example", UserDataProxy::<Example>::new)?;
        Ok(())
    }
}

fn main() {
    let file_contents = TypeWalker::new()
        //tells it that we want to generate Example
        .process_type::<Example>()
        //generate the documentation for our proxy, allowing us to
        .process_type::<UserDataProxy<Example>>()
        // enable documenting the globals
        .document_global_instance::<Export>()
        .unwrap()
        //generate the file
        .to_json()
        .expect("serde_json failed to serialize our data");

    //normally you would now save the file somewhere.
    println!("{}\n ", file_contents);

    //how you pass this type to lua hasn't changed:
    let lua = Lua::new();
    tealr::mlu::set_global_env(Export {}, &lua).unwrap();
    let globals = lua.globals();
    globals.set("test", Example { float: 42.0 }).unwrap();
    let code = "
print(\" Calling from `test` :\")
print(test:example_method(1))
print(test:example_method_mut(2,\"test\"))
print(test.example_function({}))
print(test.example_function_mut(true))
print(test.example_field)
print(test.example_static_field)
print(\" Calling from global `Example` :\")
print(Example.example_static_field)
print(Example.example_function({}))
    ";
    lua.load(code).set_name("test?").eval::<()>().unwrap();
}
