; section header
(section (section_header (section_name) @keyword))

; key value pair
(directive
    (key) @property
    (value) @string)

; comment
(comment) @comment

; variables
((value) @variable.builtin
  (#match? @variable.builtin "%[A-Za-z0-9_]+%"))

; boolean value
((value) @boolean
  (#match? @boolean "^(true|false|yes|no|on|off)$"))

; numbers
((value) @number
  (#match? @number "^[0-9]+[KMGTPkmgtp]?$"))
