<!-- Generated: 2026-03-15 | Files scanned: 32 | Token estimate: ~900 -->

# Codegen Module Codemap

## Module Tree (5,040 lines)

```
convert/
├── interface.rs    (399) RustGenerator struct + Visit impl
├── helpers.rs       (72) to_snake_case, to_pascal_case, is_string_expr
├── fn_decl.rs      (179) function declarations
├── type_mapper.rs  (257) TS→Rust type mapping
├── module.rs       (182) import/export handling
├── stmt/
│   ├── mod.rs      (158) statement dispatcher
│   ├── var_decl.rs (148) variable declarations + destructuring
│   ├── loops.rs     (96) while, for, for-of, do-while
│   ├── switch.rs    (76) switch → match
│   └── try_catch.rs(224) try-catch → Result
├── class/
│   ├── mod.rs      (375) class dispatcher + properties
│   ├── constructor (503) DI constructor + field init
│   ├── method.rs   (388) method + HTTP decorators
│   ├── getter_set. (92) get/set → accessor methods
│   ├── routing.rs  (290) Axum router + @UseGuards
│   └── mutation.rs  (30) self-mutation detection
├── expr/
│   ├── mod.rs       (96) expression dispatcher
│   ├── call.rs     (385) function/method calls
│   ├── member.rs   (135) property access
│   ├── binary.rs    (72) binary operators
│   ├── arrow.rs     (45) arrow → closure
│   ├── literal.rs  (178) object/array/template
│   └── misc.rs     (125) assign, update, optional chain
└── stdlib/
    ├── mod.rs      (130) stdlib dispatcher
    ├── array.rs    (184) 15 array methods
    ├── string.rs   (208) 16 string methods
    ├── math.rs     (130) 15 math functions
    ├── console.rs   (22) 5 console methods
    ├── json.rs      (12) stringify/parse
    └── object.rs    (22) keys/values/entries
```

## Expression Dispatch Chain

```
convert_expr(expr) → match expr {
  Lit        → convert_lit()           [literal.rs]
  Ident      → format_ident!()
  Bin        → convert_bin_expr()      [binary.rs]
  Call       → convert_call_expr()     [call.rs]
  Member     → convert_member_expr()   [member.rs]
  Arrow      → convert_arrow_expr()    [arrow.rs]
  Assign     → convert_assign_expr()   [misc.rs]
  Update     → convert_update_expr()   [misc.rs]
  Unary      → convert_unary_expr()    [misc.rs]
  Tpl        → convert_tpl()           [literal.rs]
  Object     → try_typed_object || convert_object_lit()
  Array      → convert_array_lit()     [literal.rs]
  Paren      → recurse into inner expr
  OptChain   → convert_opt_chain()     [misc.rs]
  New        → Class::new(args)
  Cond       → if/else expression
  TsAs       → no-op (strip type assertion)
  TsNonNull  → no-op (strip !)
  This       → self / state (handler context)
}
```

## Call Expression Routing

```
convert_call_expr(call) → match callee {
  console.log/error/warn   → stdlib::console
  Math.floor/sqrt/sin/...  → stdlib::math
  JSON.stringify/parse      → stdlib::json
  Object.keys/values        → stdlib::object
  array.map/filter/find/... → stdlib::array
  string.includes/split/... → stdlib::string
  axios.get/post/...        → reqwest client
  fetch()                   → reqwest::get()
  new NotFoundException()   → AppError::NotFound
  super(args)               → base field init
  Class.staticMethod()      → Class::static_method()
  generic function()        → function()
}
```

## Type Mapping (type_mapper.rs)

```
string    → String
number    → f64
boolean   → bool
void      → ()
null      → ()
any       → serde_json::Value (should be blocked)
T[]       → Vec<T>
Array<T>  → Vec<T>
Promise<T>→ Result<T, AppError>
Record<K,V> → HashMap<K,V>
Date      → String (TODO: chrono)
T | undefined → Option<T>
interface → #[derive(Serialize, Deserialize)] struct
type "a"|"b" → enum with Display
```

## NestJS Decorator Mapping

```
@Injectable()           → struct + impl (Arc<Mutex> fields for services)
@Controller('path')     → struct + router() + FromRequestParts
@Get/Post/Put/Delete()  → async handler with State<Arc<Self>>
@Body()                 → Json(body): Json<T>
@Param('id')            → Path(id): Path<String>
@Query('q')             → Query(q): Query<String>
@HttpCode(201)          → Result<(StatusCode, Json<T>), AppError>
@UseGuards(Guard)       → .layer(from_fn(guard_middleware))
@Module({...})          → parsed for DI graph (not emitted)
```
