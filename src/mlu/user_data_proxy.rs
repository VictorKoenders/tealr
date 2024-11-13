use std::marker::PhantomData;

use mlua::{AnyUserData, Error, IntoLua, Lua, UserData};

use crate::{EnumGenerator, RecordGenerator, ToTypename, Type, TypeBody, TypeName};

/// A userdata which can be used as a static proxy
pub trait StaticUserdata: UserData + 'static {}
impl<T: UserData + 'static> StaticUserdata for T {}

/// A newtype storing proxy userdata created via [`mlua::Lua::create_proxy`].
///
/// the `TypeName` for this struct is implemented as the `TypeName` for `T` concatenated with "Class".
/// For example, if your type is called "MyType", the proxy would have "MyTypeClass" for a `TypeName`.
///
/// the documentation for this proxy receives only `static` methods, i.e. those created via:
/// - `TealDataMethods::add_function`
/// - `TealDataMethods::add_meta_function`
/// - `TealDataMethods::add_meta_function_mut`
/// - `TealDataMethods::add_function_mut`
/// - `TealDataMethods::add_async_function`
/// - `TealDataFields::add_field_function_get`
/// - `TealDataFields::add_field_function_set`
/// - `TealDataFields::add_meta_field_with`
///
/// The type documentation is overriden as well.
pub struct UserDataProxy<T: StaticUserdata> {
    user_data: AnyUserData,
    ph_: PhantomData<T>,
}

impl<T: StaticUserdata> UserDataProxy<T> {
    /// Creates a new UserDataProxy
    pub fn new(lua: &Lua) -> Result<Self, Error> {
        Ok(Self {
            user_data: lua.create_proxy::<T>()?,
            ph_: Default::default(),
        })
    }
}

impl<T: StaticUserdata + ToTypename> ToTypename for UserDataProxy<T> {
    fn to_typename() -> crate::Type {
        let mut x = T::to_typename();
        if let Type::Single(x) = &mut x {
            x.name = format!("Class{}", x.name).into();
        }
        x
    }
}

impl<T: StaticUserdata + TypeBody + ToTypename> TypeBody for UserDataProxy<T> {
    fn get_type_body() -> crate::TypeGenerator {
        let generator = T::get_type_body();
        // extract only "functions"
        let type_name = Self::get_type_parts();
        let type_name_string = type_name[..type_name.len() - 1]
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join("");
        match generator {
            crate::TypeGenerator::Record(record_generator) => {
                crate::TypeGenerator::Record(Box::new(RecordGenerator {
                    // we overwrite anything which is not static
                    type_name,
                    type_doc: format!("Collection of static methods for [`{}`].", type_name_string),
                    fields: Default::default(),
                    methods: Default::default(),
                    mut_methods: Default::default(),
                    meta_method: Default::default(),
                    meta_method_mut: Default::default(),
                    ..record_generator.as_ref().clone()
                }))
            }
            crate::TypeGenerator::Enum(enum_generator) => {
                crate::TypeGenerator::Enum(EnumGenerator {
                    name: Self::get_type_parts(),
                    ..enum_generator
                })
            }
        }
    }
}

impl<T: StaticUserdata> IntoLua for UserDataProxy<T> {
    fn into_lua(self, lua: &Lua) -> mlua::Result<mlua::Value> {
        self.user_data.into_lua(lua)
    }
}
