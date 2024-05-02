def test_if_statement_inline_true
  do_something if true
end

def test_if_statement_inline_false
  do_something if false
end

def test_if_statement_inline_true_parenthesized
  do_something if (true)
end
  
def test_if_statement_inline_false_parenthesized
  do_something if (false)
end