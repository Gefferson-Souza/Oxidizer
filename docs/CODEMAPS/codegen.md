<!-- Generated: 2026-05-02 | Files scanned: 39 | Token estimate: ~1100 -->

# Codegen Module Codemap

## Module Tree (~6,000 lines, 39 files across 4 top-level modules)

```
src/
├── lib.rs              (35) public entry, ControllerMetadata, generate()
├── convert/
│   ├── mod.rs           (8) module declarations
│   ├── interface.rs   (405) RustGenerator struct + Visit impl
│   ├── helpers.rs     (118) to_snake_case, to_pascal_case, is_string_expr
│   ├── fn_decl.rs     (179) function declarations
│   ├── type_mapper.rs (269) TS→Rust type mapping (incl. Map/Set/Date)
│   ├── module.rs      (182) import/export handling
│   ├── stmt/
│   │   ├── mod.rs     (158) statement dispatcher
│   │   ├── var_decl.rs(162) variable declarations + destructuring
│   │   ├── loops.rs   (126) while, for, for-of, do-while
│   │   ├── switch.rs   (47) switch → match
│   │   └── try_catch.rs(224) try-catch → Result
│   ├── class/
│   │   ├── mod.rs     (377) class dispatcher; controller flag from registry
│   │   ├── constructor (503) DI constructor + field init
│   │   ├── method.rs  (338) method + decorator dispatch via registry
│   │   ├── getter_setter (87) get/set → accessor methods
│   │   ├── routing.rs (370) Axum router + STATUS_CODES static table + map_status_code
│   │   └── mutation.rs (60) self-mutation detection
│   └── expr/
│       ├── mod.rs     (104) expression dispatcher
│       ├── call.rs    (402) function/method calls
│       ├── member.rs  (147) property access
│       ├── binary.rs   (72) binary operators
│       ├── arrow.rs    (45) arrow → closure
│       ├── literal.rs (186) object/array/template (incl. object spread)
│       └── misc.rs    (146) assign (=/-=/*=/etc), update, optional chain
├── decorators/         — Decorator registry, ADR 0007
│   ├── mod.rs         (276) DecoratorRegistry + 3 traits + default_registry()
│   ├── controller.rs   (23) @Controller("/path")
│   ├── use_guards.rs   (22) @UseGuards(Guard1, Guard2)
│   ├── http_method.rs  (62) @Get/@Post/@Put/@Delete/@Patch (1 struct, 5 instances)
│   ├── http_code.rs    (35) @HttpCode(N)
│   └── params.rs      (128) @Body, @Param, @Query (3 structs)
└── stdlib/
    ├── mod.rs         (117) stdlib dispatcher
    ├── array.rs       (184) 15 array methods
    ├── string.rs      (208) 16 string methods
    ├── math.rs        (103) 15 math functions
    ├── console.rs      (31) 5 console methods
    ├── json.rs         (34) stringify/parse
    ├── object.rs       (46) keys/values/entries
    └── map_set.rs     (164) Map<K,V>→HashMap, Set<T>→HashSet
```

## Decorator Registry Dispatch (`src/decorators/`)

```
DecoratorRegistry (HashMap<DecoratorKind, Box<dyn _>> per scope)
  ├── class:  HashMap<DecoratorKind, Box<dyn ClassDecoratorHandler>>
  ├── method: HashMap<DecoratorKind, Box<dyn MethodDecoratorHandler>>
  └── param:  HashMap<DecoratorKind, Box<dyn ParamDecoratorHandler>>

shared_registry() → &'static DecoratorRegistry  (OnceLock-cached)

apply_class_decorators(class, &mut ClassDecoratorContext)
  populates: is_controller, controller_path, guard_names

apply_method_decorators(method, &mut MethodDecoratorContext)
  populates: http_method (Option<DecoratorKind>), route_path, http_code

first_param_decorator_kind(param) → Option<DecoratorKind>
  → param_handler(kind).emit_extractor(...) → TokenStream
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
  Assign     → convert_assign_expr()   [misc.rs]   (=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=)
  Update     → convert_update_expr()   [misc.rs]
  Unary      → convert_unary_expr()    [misc.rs]
  Tpl        → convert_tpl()           [literal.rs]
  Object     → try_typed_object || convert_object_lit()   (incl. spread)
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
  JSON.stringify/parse     → stdlib::json
  Object.keys/values       → stdlib::object
  array.map/filter/find/.. → stdlib::array
  string.includes/split/.. → stdlib::string
  map.get/set/has/...      → stdlib::map_set (HashMap)
  set.add/has/...          → stdlib::map_set (HashSet)
  Date.now()               → chrono::Utc::now().timestamp_millis()
  axios.get/post/...       → reqwest client
  fetch()                  → reqwest::get()
  new NotFoundException()  → AppError::NotFound
  super(args)              → base field init
  Class.staticMethod()     → Class::static_method()
  this.method(args)        → self.method(args)
  generic function()       → function()
}
```

## Type Mapping (type_mapper.rs)

```
string    → String
number    → f64
boolean   → bool
void      → ()
null      → ()
any       → serde_json::Value (analyzer should block)
T[]       → Vec<T>
Array<T>  → Vec<T>
Promise<T>→ Result<T, AppError>
Record<K,V> → HashMap<K,V>
Map<K,V>  → HashMap<K,V>
Set<T>    → HashSet<T>
Date      → String (TODO: chrono); Date.now() → chrono::Utc::now().timestamp_millis()
T | undefined → Option<T>
interface → #[derive(Serialize, Deserialize)] struct
type "a"|"b" → enum with Display
```

## NestJS Decorator Mapping (Registry-Based)

All NestJS decorators flow through the `decorators` registry — there is **zero** name-matching in `convert/class/*` or `convert/expr/*`. To add a new decorator, edit `tyrus_decorator_kinds::DecoratorKind` (variant + `from_name` arm) and write/register a handler. No hot-path file is touched.

```
@Injectable()           → struct + impl (Arc<Mutex> fields for services)
@Controller('path')     → ControllerHandler  → struct + router() + FromRequestParts
@Get/Post/Put/Del/Patch → HttpMethodHandler  → async handler with State<Arc<Self>>
@HttpCode(N)            → HttpCodeHandler    → Result<(StatusCode, Json<T>), AppError>
                                                StatusCode resolved via STATUS_CODES static
                                                table (compile_error! for unknown codes)
@Body()                 → BodyHandler        → axum::Json<T>
@Param('id')            → ParamHandler       → axum::extract::Path<T>
@Query('q')             → QueryHandler       → axum::extract::Query<T>
@UseGuards(Guard)       → UseGuardsHandler   → .layer(from_fn(guard_middleware))
@Module({...})          → analyzer-only (DI graph), not emitted
```
