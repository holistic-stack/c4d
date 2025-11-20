;; Keywords
[
  "module"
  "function"
  "use"
  "include"
  "for"
  "intersection_for"
  "if"
  "else"
  "let"
  "assign"
  "assert"
  "echo"
  "each"
] @keyword

;; Operators
[
  "+"
  "-"
  "*"
  "/"
  "%"
  "^"
  "!"
  "&&"
  "||"
  "?"
  ":"
  "="
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
] @operator

(modifier) @keyword.modifier


;; Punctuation
[
  ";"
  ","
  "("
  ")"
  "["
  "]"
  "{"
  "}"
] @punctuation.delimiter

;; Literals
(string) @string
(integer) @number
(float) @number
(boolean) @boolean
(undef) @constant.builtin

;; Identifiers
(identifier) @variable
(special_variable) @variable.builtin

;; Functions and Modules
(module_item
  name: (identifier) @function)

(function_item
  name: (identifier) @function)

(module_call
  name: (identifier) @function.call)

(function_call
  name: (expression) @function.call)

;; Parameters
(parameter
  (identifier) @variable.parameter)

(assignment
  name: (identifier) @variable.parameter
  (#match? @variable.parameter "^[a-zA-Z_][a-zA-Z0-9_]*$"))

;; Comments
(block_comment) @comment
(line_comment) @comment
