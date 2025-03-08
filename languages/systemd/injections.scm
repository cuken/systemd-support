; bash shell highlights
((directive
  (key) @_key
  (value) @injection.content)
 (#any-of? @_key "ExecStart" "ExecStop" "ExecReload")
 (#set! injection.language "bash"))
