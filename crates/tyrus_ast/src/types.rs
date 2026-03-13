use crate::ident::Ident;

/// All types representable in the Oxidizable Standard
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum TyrusType {
    /// `string` -> `String`
    String,
    /// `number` -> `f64`
    Number,
    /// `boolean` -> `bool`
    Boolean,
    /// `void` -> `()`
    Void,
    /// `T[]` or `Array<T>` -> `Vec<T>`
    Array(Box<TyrusType>),
    /// `T | undefined` -> `Option<T>`
    Option(Box<TyrusType>),
    /// `Record<K, V>` -> `HashMap<K, V>`
    Map(Box<TyrusType>, Box<TyrusType>),
    /// `Promise<T>` -> `Result<T, AppError>`
    Promise(Box<TyrusType>),
    /// Named type (interface, class, enum, type alias)
    Named(Ident),
    /// Generic type: `Container<T>` -> `Container<T>`
    Generic(Ident, Vec<TyrusType>),
    /// Tuple type (for destructuring)
    Tuple(Vec<TyrusType>),
    /// Inferred (when type annotation is missing)
    Inferred,
}
