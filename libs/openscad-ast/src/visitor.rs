use crate::{ast::*, Expr, Item, Stmt};

pub trait Visitor {
    fn visit_item(&mut self, item: &Item) {
        match item {
            Item::ModuleDef(n) => self.visit_stmt(&n.body),
            Item::FunctionDef(n) => self.visit_expr(&n.body),
            Item::VarDecl(_) => {}
            Item::Stmt(s) => self.visit_stmt(s),
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::UnionBlock(b) => b.items.iter().for_each(|i| self.visit_item(i)),
            Stmt::TransformChain(t) => {
                self.visit_module_call(&t.call);
                self.visit_stmt(&t.tail)
            }
            Stmt::ForBlock(f) => {
                match &f.binds {
                    ForBinds::Assigns(v) => v.iter().for_each(|a| self.visit_expr(&a.value)),
                    ForBinds::CondUpdate(c) => {
                        c.init.iter().for_each(|a| self.visit_expr(&a.value));
                        self.visit_expr(&c.cond);
                        c.update.iter().for_each(|a| self.visit_expr(&a.value));
                    }
                }
                self.visit_stmt(&f.body)
            }
            Stmt::IntersectionForBlock(f) => self.visit_stmt(&Stmt::ForBlock(f.clone())),
            Stmt::IfBlock(i) => {
                self.visit_expr(&i.condition);
                self.visit_stmt(&i.consequence);
                if let Some(a) = &i.alternative {
                    self.visit_stmt(a)
                }
            }
            Stmt::LetBlock(b) | Stmt::AssignBlock(b) => {
                b.assigns.iter().for_each(|a| self.visit_expr(&a.value));
                self.visit_stmt(&b.body)
            }
            Stmt::Include(_) | Stmt::Use(_) | Stmt::AssertStmt(_) | Stmt::Empty => {}
        }
    }

    fn visit_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(_) => {}
            Expr::Ident(_) | Expr::SpecialIdent(_) => {}
            Expr::Call { callee, args, .. } => {
                self.visit_expr(callee);
                args.iter().for_each(|a| match a {
                    Arg::Positional(e) => self.visit_expr(e),
                    Arg::Named { value, .. } => self.visit_expr(value),
                })
            }
            Expr::Index { value, index, .. } => {
                self.visit_expr(value);
                self.visit_expr(index)
            }
            Expr::DotIndex { value, .. } => self.visit_expr(value),
            Expr::Unary { expr, .. } => self.visit_expr(expr),
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right)
            }
            Expr::Ternary { cond, cons, alt, .. } => {
                self.visit_expr(cond);
                self.visit_expr(cons);
                self.visit_expr(alt)
            }
            Expr::Paren { inner, .. } => self.visit_expr(inner),
            Expr::LetExpr { assigns, body, .. } => {
                assigns.iter().for_each(|a| self.visit_expr(&a.value));
                self.visit_expr(body)
            }
            Expr::AssertExpr { args, tail, .. } | Expr::EchoExpr { args, tail, .. } => {
                args.iter().for_each(|a| match a {
                    Arg::Positional(e) => self.visit_expr(e),
                    Arg::Named { value, .. } => self.visit_expr(value),
                });
                if let Some(t) = tail {
                    self.visit_expr(t)
                }
            }
            Expr::List { cells, .. } => cells.iter().for_each(|c| match c {
                ListCell::Expr(e) => self.visit_expr(e),
            }),
            Expr::Range { start, inc, end, .. } => {
                self.visit_expr(start);
                if let Some(i) = inc {
                    self.visit_expr(i)
                }
                self.visit_expr(end)
            }
            Expr::FunctionLit { params: _, body, .. } => self.visit_expr(body),
            Expr::Each { value, .. } => self.visit_expr(value),
            Expr::ListComp { kind, .. } => match &**kind {
                ListCompKind::For { binds, cell } => {
                    match &**binds {
                        ForBinds::Assigns(v) => v.iter().for_each(|a| self.visit_expr(&a.value)),
                        ForBinds::CondUpdate(c) => {
                            c.init.iter().for_each(|a| self.visit_expr(&a.value));
                            self.visit_expr(&c.cond);
                            c.update.iter().for_each(|a| self.visit_expr(&a.value));
                        }
                    }
                    self.visit_expr(cell)
                }
                ListCompKind::If { condition, consequence, alternative } => {
                    self.visit_expr(condition);
                    self.visit_expr(consequence);
                    if let Some(a) = alternative { self.visit_expr(a) }
                }
            },
        }
    }

    fn visit_module_call(&mut self, _mc: &ModuleCall) {}
}