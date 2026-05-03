use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitWith};
use tyrus_decorator_kinds::DecoratorKind;
use tyrus_di::graph::DiGraph;
use tyrus_di::module::Module;
use tyrus_di::provider::{InjectionScope, Provider};

pub struct DecoratorVisitor {
    pub graph: DiGraph,
    pub current_module: Option<Module>,
}

impl DecoratorVisitor {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            current_module: None,
        }
    }
}

impl Default for DecoratorVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl DecoratorVisitor {
    fn extract_decorator_args(&self, decorator: &Decorator) -> Option<ObjectLit> {
        if let Expr::Call(call_expr) = &*decorator.expr {
            if let Some(arg) = call_expr.args.first() {
                if let Expr::Object(obj_lit) = &*arg.expr {
                    return Some(obj_lit.clone());
                }
            }
        }
        None
    }

    /// Parses an `@Module({ imports, providers, controllers, exports })`
    /// decorator into a [`Module`] for the DI graph.
    fn parse_module_metadata(&self, class_name: &str, decorator: &Decorator) -> Module {
        let mut module = Module {
            name: class_name.to_string(),
            imports: vec![],
            providers: vec![],
            controllers: vec![],
            exports: vec![],
        };
        let Some(obj) = self.extract_decorator_args(decorator) else {
            return module;
        };
        for prop in obj.props {
            let PropOrSpread::Prop(prop) = prop else {
                continue;
            };
            let Prop::KeyValue(kv) = *prop else { continue };
            let PropName::Ident(key) = kv.key else {
                continue;
            };
            let Expr::Array(arr) = *kv.value else {
                continue;
            };
            let values = collect_ident_array(&arr);
            match key.sym.as_ref() {
                "imports" => module.imports = values,
                "providers" => module.providers = values_to_providers(values),
                "controllers" => module.controllers = values,
                "exports" => module.exports = values,
                _ => {}
            }
        }
        module
    }
}

/// Collects ident expressions from an array literal, ignoring spreads/non-idents.
fn collect_ident_array(arr: &ArrayLit) -> Vec<String> {
    arr.elems
        .iter()
        .filter_map(|e| {
            let ExprOrSpread { expr, .. } = e.as_ref()?;
            if let Expr::Ident(i) = &**expr {
                return Some(i.sym.to_string());
            }
            None
        })
        .collect()
}

/// Wraps each ident name in a `Provider` (singleton, deps filled later).
fn values_to_providers(values: Vec<String>) -> Vec<Provider> {
    values
        .into_iter()
        .map(|v| Provider {
            token: v.clone(),
            implementation: v,
            scope: InjectionScope::Singleton,
            dependencies: vec![],
        })
        .collect()
}

impl Visit for DecoratorVisitor {
    fn visit_class_decl(&mut self, n: &ClassDecl) {
        let class_name = n.ident.sym.to_string();
        let mut is_injectable = false;

        for decorator in &n.class.decorators {
            let Expr::Call(call_expr) = &*decorator.expr else {
                continue;
            };
            let Callee::Expr(callee_expr) = &call_expr.callee else {
                continue;
            };
            let Expr::Ident(ident) = &**callee_expr else {
                continue;
            };
            // Single source of truth for decorator name → kind classification.
            // Adding a new decorator requires extending tyrus_decorator_kinds,
            // not this match arm.
            let Some(kind) = DecoratorKind::from_name(ident.sym.as_ref()) else {
                continue;
            };
            match kind {
                DecoratorKind::Module => {
                    let module = self.parse_module_metadata(&class_name, decorator);
                    self.graph.add_module(module);
                }
                DecoratorKind::Injectable | DecoratorKind::Controller => {
                    is_injectable = true;
                }
                _ => {}
            }
        }

        if is_injectable {
            // Extract dependencies from constructor
            let mut deps = vec![];
            for member in &n.class.body {
                if let ClassMember::Constructor(cons) = member {
                    for param in &cons.params {
                        match param {
                            ParamOrTsParamProp::TsParamProp(prop) => {
                                if let TsParamPropParam::Ident(ident) = &prop.param {
                                    if let Some(type_ann) = &ident.type_ann {
                                        if let Some(type_ref) = type_ann.type_ann.as_ts_type_ref() {
                                            if let Some(type_name) = type_ref.type_name.as_ident() {
                                                deps.push(type_name.sym.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                            ParamOrTsParamProp::Param(param) => {
                                if let Pat::Ident(ident) = &param.pat {
                                    if let Some(type_ann) = &ident.type_ann {
                                        if let Some(type_ref) = type_ann.type_ann.as_ts_type_ref() {
                                            if let Some(type_name) = type_ref.type_name.as_ident() {
                                                deps.push(type_name.sym.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Add to graph as definition
            self.graph
                .add_injectable(tyrus_di::provider::InjectableDefinition {
                    name: class_name.clone(),
                    dependencies: deps,
                    scope: InjectionScope::Singleton,
                });
        }
        n.visit_children_with(self);
    }
}
