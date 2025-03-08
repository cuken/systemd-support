(section_header
    (section_name) @redact)

((directive
  (key) @_key
  (value) @redact)
 (#any-of? @_key "Password" "Secret" "Key" "Token" "ApiKey" "PrivateKey"))
