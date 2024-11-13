use tealr::{
    mlu::{TealData, TealDataMethods, TypedFunction, UserData},
    type_parts_to_str, ToTypename,
};
#[test]
fn generate_correct_type() {
    assert_eq!(
        type_parts_to_str(
            #[allow(deprecated)]
            TypedFunction::<String, String>::to_old_type_parts()
        ),
        "function(string):(string)"
    );
    assert_eq!(
        type_parts_to_str(
            #[allow(deprecated)]
            TypedFunction::<TypedFunction::<(i8, String), (String, u8)>, f32>::to_old_type_parts()
        ),
        "function(function(integer , string):(string , integer)):(number)"
    );
}
#[test]
fn try_to_use() -> mlua::Result<()> {
    #[derive(Clone, UserData, ToTypename)]
    struct Test {}
    impl TealData for Test {
        fn add_methods<T: TealDataMethods<Self>>(methods: &mut T) {
            methods.add_method(
                "test_function_as_parameter",
                |_, _, x: TypedFunction<(u8, u8), u8>| x.call((10, 20)),
            );
        }
    }
    let code = tealr::compile_inline_teal!(
        "
global record Test
    test_function_as_parameter:function(Test,function(integer,integer):integer):integer
end

global test: Test

local function add(a:integer,b:integer):integer
    return a + b
end
return test:test_function_as_parameter(add)
"
    );
    let lua = mlua::Lua::new();
    let globals = lua.globals();
    globals.set("test", Test {})?;
    let res: i32 = lua.load(code).eval()?;
    assert_eq!(res, 30);
    Ok(())
}

#[test]
fn pass_back() -> mlua::Result<()> {
    #[derive(Clone, UserData, ToTypename)]
    struct Test {}
    impl TealData for Test {
        fn add_methods<T: TealDataMethods<Self>>(methods: &mut T) {
            methods.add_method(
                "test_function_as_parameter",
                |_, _, x: TypedFunction<(u8, u8), u8>| Ok(x),
            );
        }
    }
    let code = tealr::compile_inline_teal!(
        "
global record Test
    test_function_as_parameter:function(Test,function(integer,integer):integer):(function(integer,integer):integer)
end

global test: Test

global function add(a:integer,b:integer):integer
    return a + b
end
return test:test_function_as_parameter(add)(10,20)
"
    );

    let lua = mlua::Lua::new();
    let globals = lua.globals();
    globals.set("test", Test {})?;
    let res: i32 = lua.load(code).eval()?;
    assert_eq!(res, 30);
    Ok(())
}
