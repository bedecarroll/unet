# DSL Syntax Reference

This document provides a complete reference for the μNet Policy Domain Specific Language (DSL) syntax.

## Grammar Overview

The μNet Policy DSL uses a formal grammar defined in Pest (Parser Expression Grammar). The complete grammar is available at `crates/unet-core/src/policy/policy.pest`.

### Basic Structure

```
PolicyRule := "WHEN" Condition "THEN" Action
```

Every policy rule consists of exactly one condition and one action separated by the keywords `WHEN` and `THEN`.

## Lexical Elements

### Keywords

| Keyword | Context | Description |
|---------|---------|-------------|
| `WHEN` | Rule start | Begins the condition portion |
| `THEN` | Rule middle | Separates condition from action |
| `SET` | Action | Assigns a value to a field |
| `TO` | Action | Follows SET to specify target value |
| `ASSERT` | Action | Validates a field value |
| `IS` | Action | Follows ASSERT to specify expected value |
| `APPLY` | Action | Assigns a template to a node |
| `AND` | Condition | Logical AND operator |
| `OR` | Condition | Logical OR operator |
| `NOT` | Condition | Logical NOT operator |
| `NULL` | Value | Represents null/empty value |
| `CONTAINS` | Operator | String containment check |
| `MATCHES` | Operator | Regular expression match |

### Operators

#### Comparison Operators

| Operator | Symbol | Description | Example |
|----------|--------|-------------|---------|
| Equal | `==` | Exact equality | `node.vendor == "cisco"` |
| Not Equal | `!=` | Not equal | `node.role != "switch"` |
| Less Than | `<` | Numeric less than | `node.port_count < 24` |
| Less Than or Equal | `<=` | Numeric less than or equal | `custom_data.cpu <= 80` |
| Greater Than | `>` | Numeric greater than | `custom_data.uptime > 86400` |
| Greater Than or Equal | `>=` | Numeric greater than or equal | `node.port_count >= 48` |
| Contains | `CONTAINS` | String containment | `node.model CONTAINS "2960"` |
| Matches | `MATCHES` | Regex pattern match | `node.name MATCHES "^rtr-.*"` |

#### Logical Operators

| Operator | Precedence | Associativity | Example |
|----------|------------|---------------|---------|
| `NOT` | Highest | Right | `NOT (node.enabled == false)` |
| `AND` | Medium | Left | `node.vendor == "cisco" AND node.role == "switch"` |
| `OR` | Lowest | Left | `node.vendor == "cisco" OR node.vendor == "arista"` |

#### Existence Operators

| Operator | Description | Example |
|----------|-------------|---------|
| `IS NULL` | Field is null/missing | `custom_data.location IS NULL` |
| `IS NOT NULL` | Field exists and has value | `node.management_ip IS NOT NULL` |

### Identifiers

Field references use dot notation to access nested properties:

```
field_reference := identifier ("." identifier)*
```

Examples:
- `node.vendor`
- `custom_data.location.rack`
- `derived.interfaces.eth0.status`

#### Reserved Field Prefixes

| Prefix | Description | Source |
|--------|-------------|---------|
| `node.` | Node properties | Database node record |
| `custom_data.` | Custom data fields | JSON field in node record |
| `derived.` | Derived/computed data | SNMP polling, discovery |

### Literals

#### String Literals

Strings must be enclosed in double quotes:

```
string_literal := "\"" char* "\""
```

Examples:
- `"cisco"`
- `"datacenter-1"`
- `"192.168.1.1"`

**Escape Sequences:**
- `\"` - Double quote
- `\\` - Backslash
- `\n` - Newline
- `\t` - Tab
- `\r` - Carriage return

#### Numeric Literals

Numbers can be integers or decimals:

```
number := integer | decimal
integer := digit+
decimal := digit+ "." digit+
```

Examples:
- `24`
- `100.5`
- `-1`
- `0.0`

#### Boolean Literals

Boolean values are case-sensitive:

```
boolean := "true" | "false"
```

#### Null Literal

Represents absence of value:

```
null := "null"
```

### Regular Expressions

Regex patterns are specified as string literals in `MATCHES` operations:

```
node.name MATCHES "^(router|rtr)-[0-9]+"
```

The regex engine supports standard PCRE syntax.

## Conditions

Conditions are boolean expressions that determine when an action should execute.

### Grammar

```
Condition := OrExpression
OrExpression := AndExpression ("OR" AndExpression)*
AndExpression := NotExpression ("AND" NotExpression)*
NotExpression := "NOT"? ComparisonExpression
ComparisonExpression := FieldReference ComparisonOperator Value
                     | FieldReference "IS" "NULL"
                     | FieldReference "IS" "NOT" "NULL"
                     | "(" Condition ")"
```

### Precedence Rules

1. **Parentheses** - Highest precedence
2. **NOT** - Unary negation
3. **Comparison operators** - `==`, `!=`, `<`, `<=`, `>`, `>=`, `CONTAINS`, `MATCHES`
4. **AND** - Logical conjunction  
5. **OR** - Logical disjunction (lowest precedence)

### Examples

#### Simple Conditions

```
node.vendor == "cisco"
custom_data.port_count > 24
node.management_ip IS NOT NULL
```

#### Complex Conditions

```
(node.vendor == "cisco" AND node.role == "switch") OR (node.vendor == "arista" AND node.model CONTAINS "7050")

NOT (node.lifecycle == "decommissioned") AND custom_data.monitoring.enabled == true

node.name MATCHES "^(core|dist)-.*" AND custom_data.location.datacenter == "dc1"
```

## Actions

Actions define operations to perform when conditions are satisfied.

### Grammar

```
Action := SetAction | AssertAction | ApplyAction

SetAction := "SET" FieldReference "TO" Value
AssertAction := "ASSERT" FieldReference "IS" Value  
ApplyAction := "APPLY" StringLiteral
```

### SET Action

Updates custom_data fields with new values.

#### Syntax

```
SET <field_reference> TO <value>
```

#### Constraints

- Field reference must start with `custom_data.`
- Creates nested objects as needed
- Overwrites existing values

#### Examples

```
SET custom_data.location TO "datacenter-1"
SET custom_data.snmp.community TO "private"
SET custom_data.monitoring.enabled TO true
SET custom_data.thresholds.cpu_warning TO 75.5
```

### ASSERT Action

Validates that a field matches an expected value.

#### Syntax

```
ASSERT <field_reference> IS <value>
```

#### Behavior

- Reads field value and compares to expected value
- Succeeds if values match exactly
- Fails if values don't match or field doesn't exist
- Used for compliance validation

#### Examples

```
ASSERT node.version IS "15.1"
ASSERT custom_data.security.ssh_enabled IS true
ASSERT custom_data.backup.frequency IS "daily"
```

### APPLY Action

Assigns configuration templates to nodes.

#### Syntax

```
APPLY "<template_path>"
```

#### Behavior

- Adds template path to node's `assigned_templates` list
- Templates are applied in order during configuration generation
- Duplicate assignments are ignored

#### Examples

```
APPLY "templates/base-config.j2"
APPLY "templates/cisco/switch-base.j2"
APPLY "templates/security/hardening.j2"
```

## Data Types and Conversions

### Type System

The DSL has a simple type system for values:

| Type | Rust Type | JSON Type | Examples |
|------|-----------|-----------|----------|
| String | `String` | `string` | `"cisco"`, `"10.1.1.1"` |
| Number | `f64` | `number` | `24`, `100.5`, `-1` |
| Boolean | `bool` | `boolean` | `true`, `false` |
| Null | `Option<T>` | `null` | `null` |

### Type Coercion

The evaluator performs automatic type coercion for comparisons:

#### String Comparisons

- Numbers are converted to strings: `24` becomes `"24"`
- Booleans are converted to strings: `true` becomes `"true"`
- Null becomes empty string: `null` becomes `""`

#### Numeric Comparisons

- Strings containing valid numbers are parsed: `"24"` becomes `24`
- Boolean conversion: `true` = `1`, `false` = `0`
- Null becomes `0`

#### Boolean Comparisons

- Strings: `"true"` = `true`, `"false"` = `false`, others = `false`
- Numbers: `0` = `false`, non-zero = `true`
- Null = `false`

### Field Resolution

Field references are resolved against the evaluation context:

```rust
struct EvaluationContext {
    pub node_data: JsonValue,
    pub derived_data: Option<JsonValue>,
}
```

#### Resolution Order

1. Check `derived_data` for `derived.*` fields
2. Check `node_data` for all other fields
3. Return `null` if field not found

#### Nested Field Access

Fields are accessed using JSON pointer semantics:

- `custom_data.location.rack` → `node_data["custom_data"]["location"]["rack"]`
- Missing intermediate objects return `null`
- Array indices not supported in current version

## Comments and Whitespace

### Comments

The DSL supports line comments starting with `//`:

```
// This is a comment
WHEN node.vendor == "cisco" // End-of-line comment
THEN SET custom_data.vendor_class TO "ios"
```

### Whitespace

- Whitespace is ignored between tokens
- Newlines separate policy rules
- Indentation has no semantic meaning

### Multi-line Rules

Rules can span multiple lines:

```
WHEN node.vendor == "cisco" 
     AND node.role == "switch" 
     AND custom_data.location.datacenter == "dc1"
THEN SET custom_data.config_template TO "cisco-switch-dc1.j2"
```

## Error Handling

### Parse Errors

Parse errors occur when syntax is invalid:

```
ParseError::UnexpectedToken {
    expected: "THEN",
    found: "AND",
    line: 5,
    column: 20
}
```

### Evaluation Errors

Evaluation errors occur during rule execution:

#### Field Not Found

```
PolicyError::FieldNotFound { 
    field: "custom_data.nonexistent.field" 
}
```

#### Type Mismatch

```
PolicyError::TypeMismatch { 
    expected: "String", 
    actual: "Number" 
}
```

#### Invalid Regex

```
PolicyError::InvalidRegex { 
    pattern: "[invalid-pattern" 
}
```

## Performance Considerations

### Optimization Guidelines

1. **Simple conditions first**: Put most selective conditions early
2. **Avoid complex regex**: Use simple string operations when possible
3. **Minimize field access**: Cache commonly accessed values
4. **Use appropriate operators**: `CONTAINS` is faster than regex for simple substring matching

### Grammar Complexity

The parser has linear time complexity O(n) for well-formed input, where n is the input length.

### Memory Usage

- Each parsed rule consumes ~200 bytes of memory
- Field references are interned to reduce duplication
- Large policy files should be split when possible

## Reserved Words

These identifiers cannot be used as field names:

```
WHEN, THEN, SET, TO, ASSERT, IS, APPLY, AND, OR, NOT, 
NULL, CONTAINS, MATCHES, true, false
```

## Formal Grammar

The complete formal grammar in Pest syntax:

```pest
// Policy file contains multiple rules
policy_file = { SOI ~ (rule ~ NEWLINE*)* ~ EOI }

// A rule has a condition and an action
rule = { "WHEN" ~ condition ~ "THEN" ~ action }

// Conditions use logical operators
condition = { or_expr }
or_expr = { and_expr ~ ("OR" ~ and_expr)* }
and_expr = { not_expr ~ ("AND" ~ not_expr)* }
not_expr = { "NOT"? ~ primary_expr }

primary_expr = { 
    comparison | 
    existence_check | 
    "(" ~ condition ~ ")" 
}

comparison = { 
    field_ref ~ comparison_op ~ value 
}

existence_check = { 
    field_ref ~ "IS" ~ "NULL" |
    field_ref ~ "IS" ~ "NOT" ~ "NULL"
}

comparison_op = {
    "==" | "!=" | "<=" | ">=" | "<" | ">" | 
    "CONTAINS" | "MATCHES"
}

// Actions
action = { set_action | assert_action | apply_action }

set_action = { "SET" ~ field_ref ~ "TO" ~ value }
assert_action = { "ASSERT" ~ field_ref ~ "IS" ~ value }
apply_action = { "APPLY" ~ string_literal }

// Field references and values
field_ref = { identifier ~ ("." ~ identifier)* }
identifier = { (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }

value = { 
    string_literal | 
    number_literal | 
    boolean_literal | 
    null_literal 
}

string_literal = { "\"" ~ string_content ~ "\"" }
string_content = { (!("\"" | "\\") ~ ANY | escape_sequence)* }
escape_sequence = { "\\" ~ ("\"" | "\\" | "n" | "t" | "r") }

number_literal = { "-"? ~ (decimal | integer) }
decimal = { ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT+ }
integer = { ASCII_DIGIT+ }

boolean_literal = { "true" | "false" }
null_literal = { "null" }

// Whitespace and comments
WHITESPACE = _{ " " | "\t" }
COMMENT = _{ "//" ~ (!NEWLINE ~ ANY)* }
```

This grammar ensures that the DSL is both human-readable and machine-parseable with unambiguous syntax.