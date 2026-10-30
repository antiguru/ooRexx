p = .Yaml~new
yaml1 = '---' || "0A"x || 'key: [a,' || "0A"x || '  b, c]' || "0A"x
doc1 = p~parseString(yaml1)
say doc1["key"]~items
::requires 'yaml.cls'
