use glsl_lang::ast::FunIdentifierData::TypeSpecifier;
use glsl_lang::ast::{
    AssignmentOp, AssignmentOpData, BinaryOpData, Declaration, DeclarationData, Expr, ExprData,
    ExternalDeclarationData, FunIdentifier, FunIdentifierData, FunctionDefinition,
    FunctionParameterDeclarationData, IdentifierData, PreprocessorDefine, PreprocessorDefineData,
    StorageQualifierData, TranslationUnit, TypeQualifierSpecData, TypeSpecifierNonArrayData,
};
use glsl_lang::parse::DefaultParse;
use glsl_lang::transpiler::glsl;
use glsl_lang::transpiler::glsl::{FormattingSettings, FormattingState, IndentStyle, Whitespace};
use glsl_lang::visitor::{HostMut, Visit, VisitorMut};
use once_cell::sync::Lazy;
use smol_str::SmolStr;
use std::collections::HashMap;

use crate::profile_scope;

#[derive(Default)]
pub struct Minifier {
    function_mapping: HashMap<String, String>,
    local_mapping: HashMap<String, String>,
}


static FORMATTING_STATE: Lazy<FormattingState> = Lazy::new(|| {
    let mut formatting_state = FormattingState::default();
    formatting_state.settings = &FormattingSettings {
        indent_style: IndentStyle::None,
        space_before_open_block: false,
        newline_after_open_block: false,
        newline_before_close_block: false,
        newline_after_close_block: false,
        newline_after_collapsed_statement: false,
        newline_before_collapsed_statement: false,
        struct_field_separator: Whitespace::None,
        struct_declaration_terminator: Whitespace::None,
        declaration_terminator: Whitespace::None,
        case_label_terminator: Whitespace::None,
        spaces_around_binary_ops: false,
        statement_terminator: Whitespace::None,
        function_definition_terminator: Whitespace::None,
        collapse_single_item_compound_statements: false,
        space_before_else: false,
        space_after_list_separator: false,
        space_after_for_statement_separator: false,
        spaces_surrounding_initializer_list_expressions: false,
        spaces_surrounding_statement_parentheses: false,
    };
    formatting_state
});

impl Minifier {
    pub fn minify(
        &mut self,
        source: &str,
        rename: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        profile_scope!("minify_shader");
        let mut ast =
            TranslationUnit::parse(source).map_err(|e| format!("GLSL Parse Error: {}", e))?;

        // Todo: Fix this, it is broken with ItemsAdder shaders
        if rename {
            self.generate_function_mapping(&ast);
            self.collect_all_locals(&mut ast);

            ast.visit_mut(self);
        }

        let mut output = String::new();
        glsl::show_translation_unit(&mut output, &ast, *FORMATTING_STATE)?;
        Ok(optimize_string(output.as_mut_str()))
    }

    fn generate_function_mapping(&mut self, ast: &TranslationUnit) {
        let mut count = 0;
        for ext_decl in &ast.0 {
            if let ExternalDeclarationData::FunctionDefinition(fd) = &ext_decl.content {
                let name = &fd.prototype.name.0.to_string();

                // Skip main
                if name != "main" && !self.function_mapping.contains_key(name) {
                    let new_name = generate_short_name(count);
                    self.function_mapping.insert(name.clone(), new_name);
                    count += 1;
                }
            }
        }
    }

    fn collect_all_locals(&mut self, ast: &mut TranslationUnit) {
        let mut collector = NameCollector { minifier: self };
        ast.visit_mut(&mut collector);
    }

    fn register_local(&mut self, name: String) {
        if !self.local_mapping.contains_key(&name) {
            let new_name = generate_short_name(self.local_mapping.len());
            self.local_mapping.insert(name, new_name);
        }
    }
}

struct NameCollector<'a> {
    minifier: &'a mut Minifier,
}

impl<'a> VisitorMut for NameCollector<'a> {
    fn visit_function_definition(&mut self, fd: &mut FunctionDefinition) -> Visit {
        for param in &mut fd.prototype.parameters {
            if let FunctionParameterDeclarationData::Named(_, declarator) = &mut param.content {
                let p_name = declarator.ident.ident.0.to_string();
                self.minifier.register_local(p_name);
            }
        }
        Visit::Children
    }

    fn visit_declaration(&mut self, decl: &mut Declaration) -> Visit {
        if let DeclarationData::InitDeclaratorList(list) = &mut decl.content {
            let is_storage = if let Some(qualifier) = &list.head.ty.content.qualifier {
                qualifier
                    .content
                    .qualifiers
                    .iter()
                    .any(|q| matches!(q.content, TypeQualifierSpecData::Storage(_)))
            } else {
                false
            };

            if !is_storage {
                if let Some(ref name_node) = list.head.name {
                    let name = name_node.0.to_string();
                    if !name.starts_with("gl_") {
                        self.minifier.register_local(name);
                    }
                }
                for tail in &mut list.tail {
                    let t_name = tail.ident.ident.0.to_string();
                    if !t_name.starts_with("gl_") {
                        self.minifier.register_local(t_name);
                    }
                }
            }
        }
        Visit::Children
    }
}

impl VisitorMut for Minifier {
    fn visit_function_definition(&mut self, fd: &mut FunctionDefinition) -> Visit {
        let func_name = fd.prototype.name.0.to_string();
        if let Some(new_name) = self.function_mapping.get(&func_name) {
            fd.prototype.name.0 = SmolStr::new(new_name);
        }

        // Because of `#define function()` we cannot have a different mapping for each function
        //self.local_mapping.clear();

        for param in &mut fd.prototype.parameters {
            if let FunctionParameterDeclarationData::Named(_, declarator) = &mut param.content {
                let p_name = declarator.ident.ident.0.to_string();
                if let Some(short) = self.local_mapping.get(&p_name) {
                    declarator.ident.ident.0 = SmolStr::new(short);
                }
            }
        }

        Visit::Children
    }

    fn visit_preprocessor_define(&mut self, define: &mut PreprocessorDefine) -> Visit {
        match &mut define.content {
            PreprocessorDefineData::FunctionLike { ident, args, value } => {
                let macro_name = ident.0.to_string();
                if let Some(short) = self.local_mapping.get(&macro_name) {
                    ident.0 = SmolStr::new(short);
                }

                for arg in args {
                    let arg_name = arg.0.to_string();
                    if let Some(short) = self.local_mapping.get(&arg_name) {
                        arg.0 = SmolStr::new(short);
                    }
                }
                let mut new_value = value.clone();

                let mut all_maps: Vec<_> = self.local_mapping.iter().collect();
                all_maps.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

                for (old, short) in all_maps {
                    if let Ok(re) = regex::Regex::new(&format!(r"\b{}\b", old)) {
                        new_value = re.replace_all(&new_value, short.as_str()).into_owned();
                    }
                }
                *value = new_value;
            }
            PreprocessorDefineData::ObjectLike { ident, value } => {
                let old_name = ident.0.to_string();
                if let Some(short) = self.local_mapping.get(&old_name) {
                    ident.0 = SmolStr::new(short);
                }

                let mut new_value = value.clone();
                let mut all_maps: Vec<_> = self.local_mapping.iter().collect();
                all_maps.sort_by_key(|b| std::cmp::Reverse(b.0.len()));

                for (old, short) in all_maps {
                    if let Ok(re) = regex::Regex::new(&format!(r"\b{}\b", old)) {
                        new_value = re.replace_all(&new_value, short.as_str()).into_owned();
                    }
                }
                *value = new_value;
            }
        }
        Visit::Children
    }

    fn visit_declaration(&mut self, decl: &mut Declaration) -> Visit {
        if let DeclarationData::InitDeclaratorList(list) = &mut decl.content {
            let has_forbidden_qualifier = if let Some(qualifier) = &list.head.ty.content.qualifier {
                qualifier.content.qualifiers.iter().any(|q| {
                    if let TypeQualifierSpecData::Storage(storage_qual) = &q.content {
                        matches!(
                            storage_qual.content,
                            StorageQualifierData::Uniform
                                | StorageQualifierData::In
                                | StorageQualifierData::Out
                                | StorageQualifierData::Attribute
                                | StorageQualifierData::Varying
                                | StorageQualifierData::Const
                        )
                    } else {
                        false
                    }
                })
            } else {
                false
            };
            if !has_forbidden_qualifier {
                let name = list.head.name.clone().unwrap().0.to_string();

                if !name.starts_with("gl_") {
                    if let Some(short) = self.local_mapping.get(&name) {
                        list.head.name = Some(IdentifierData::from(short.as_str()).into());
                    }
                }

                for tail in &mut list.tail {
                    let t_name = tail.ident.ident.0.to_string();
                    if !t_name.starts_with("gl_") {
                        if let Some(short) = self.local_mapping.get(&t_name) {
                            tail.ident.ident.0 = SmolStr::new(short);
                        }
                    }
                }
            }
        }
        Visit::Children
    }

    fn visit_expr(&mut self, expr: &mut Expr) -> Visit {
        if let ExprData::Variable(ident) = &mut expr.content {
            let var_name = ident.0.to_string();
            if let Some(short) = self.local_mapping.get(&var_name) {
                ident.0 = SmolStr::new(short);
            }
        }

        // minify `vec4(1.0, 1.0, 1.0, 1.0)` to `vec(1.)`
        if let ExprData::FunCall(ident, args) = &mut expr.content {
            if let Some(name_node) = get_fun_name(ident) {
                let name = name_node.to_string();

                let expected_len = match name.as_str() {
                    "vec2" | "ivec2" | "uvec2" | "bvec2" => Some(2),
                    "vec3" | "ivec3" | "uvec3" | "bvec3" => Some(3),
                    "vec4" | "ivec4" | "uvec4" | "bvec4" => Some(4),
                    _ => None,
                };

                if let Some(len) = expected_len {
                    if args.len() == len {
                        let first_val = if let Some(first_arg) = args.first() {
                            if let ExprData::FloatConst(f) = &first_arg.content {
                                Some(*f)
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        if let Some(val) = first_val {
                            let all_identical = args.iter().all(|arg| {
                                if let ExprData::FloatConst(f) = &arg.content {
                                    *f == val
                                } else {
                                    false
                                }
                            });

                            if all_identical {
                                args.truncate(1);
                            }
                        }
                    }
                }
            }
        }

        // minify `a = a * a` to `a *= a`
        if let ExprData::Assignment(ref mut lhs, ref op, ref mut rhs) = expr.content {
            if let AssignmentOpData::Equal = op.content {
                if let ExprData::Binary(ref mut bin_op, ref mut bin_lhs, ref mut bin_rhs) =
                    rhs.content
                {
                    if lhs == bin_lhs {
                        let new_op = match bin_op.content {
                            BinaryOpData::Add => Some(AssignmentOpData::Add),
                            BinaryOpData::Sub => Some(AssignmentOpData::Sub),
                            BinaryOpData::Mult => Some(AssignmentOpData::Mult),
                            BinaryOpData::Div => Some(AssignmentOpData::Div),
                            BinaryOpData::Mod => Some(AssignmentOpData::Mod),
                            BinaryOpData::LShift => Some(AssignmentOpData::LShift),
                            BinaryOpData::RShift => Some(AssignmentOpData::RShift),
                            BinaryOpData::BitAnd => Some(AssignmentOpData::And),
                            BinaryOpData::BitXor => Some(AssignmentOpData::Xor),
                            BinaryOpData::BitOr => Some(AssignmentOpData::Or),
                            _ => None,
                        };

                        if let Some(op_replacement) = new_op {
                            let mut new_rhs_node = Expr::new(ExprData::IntConst(0), None);
                            std::mem::swap(&mut new_rhs_node, bin_rhs);
                            *expr = Expr::new(
                                ExprData::Assignment(
                                    lhs.clone(),
                                    AssignmentOp::new(op_replacement, None),
                                    Box::new(new_rhs_node),
                                ),
                                None,
                            );
                        }
                    }
                }
            }
        }

        // minify `vec.rgba` to `vec.xyzw` (useful for compression in zip files to reduce the number of different characters)
        if let ExprData::Dot(ref _sub_expr, ref mut field) = expr.content {
            let field_name = field.0.to_string();

            if !field_name.is_empty() && field_name.chars().all(|c| "rgbastpq".contains(c)) {
                // check if it is rgba or stpq
                let new_swizzle = normalize_swizzle(&field_name);
                field.0 = SmolStr::new(new_swizzle);
            }
        }

        Visit::Children
    }

    fn visit_fun_identifier(&mut self, ident: &mut FunIdentifier) -> Visit {
        match &mut ident.content {
            TypeSpecifier(ts) => {
                if let TypeSpecifierNonArrayData::TypeName(tn) = &mut ts.content.ty.content {
                    let old_name = tn.0.to_string();
                    if let Some(new_name) = self.function_mapping.get(&old_name) {
                        tn.0 = SmolStr::new(new_name);
                    }
                }
            }
            FunIdentifierData::Expr(expr) => {
                if let ExprData::Variable(var_ident) = &mut expr.content {
                    let old_name = var_ident.0.to_string();
                    if let Some(new_name) = self.function_mapping.get(&old_name) {
                        var_ident.0 = SmolStr::new(new_name);
                    }
                }
            }
        }
        Visit::Children
    }
}

fn get_fun_name(ident: &FunIdentifier) -> Option<String> {
    match &ident.content {
        TypeSpecifier(ts) => match &ts.content.ty.content {
            TypeSpecifierNonArrayData::TypeName(tn) => Some(tn.0.to_string()),
            other => {
                let debug_name = format!("{:?}", other);
                Some(debug_name.to_lowercase())
            }
        },
        FunIdentifierData::Expr(expr) => {
            if let ExprData::Variable(var_ident) = &expr.content {
                Some(var_ident.0.to_string())
            } else {
                None
            }
        }
    }
}

fn generate_short_name(mut id: usize) -> String {
    let mut name = String::new();
    loop {
        let rem = id % 26;
        name.push((b'a' + rem as u8) as char);
        id /= 26;
        if id == 0 {
            break;
        }
    }
    name
}

fn optimize_string(input: &mut str) -> String {
    input
        // remove necessary spaces that are still here after converting the AST back to string
        .replace(" {", "{")
        .replace("{ ", "{")
        .replace(" }", "}")
        .replace(" (", "(")
        .replace("( ", "(")
        .replace(") ", ")")
        .replace(" )", ")")
        .replace(", ", ",")
        // rust version of Regex::new(r"[\t ]+").unwrap().replace_all(&input, " ");
        .split([' ', '\t'])
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn normalize_swizzle(swizzle: &str) -> String {
    swizzle
        .chars()
        .map(|c| match c {
            'r' | 's' => 'x',
            'g' | 't' => 'y',
            'b' | 'p' => 'z',
            'a' | 'q' => 'w',
            _ => c,
        })
        .collect()
}
