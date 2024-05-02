def test_unless_statement_inline_true
  do_something unless true
end

def test_unless_statement_inline_false
  do_something unless false
end

def test_unless_statement_inline_true_parenthesized
  do_something unless (true)
end
  
def test_unless_statement_inline_false_parenthesized
  do_something unless (false)
end