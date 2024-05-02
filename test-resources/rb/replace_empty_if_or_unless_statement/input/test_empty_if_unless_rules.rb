def test_empty_unless_statement_true
  unless true
  end
end

def test_empty_unless_statement_false
  unless false
  end
end

def test_empty_unless_statement_parenthesized_true
  unless(true)
  end
end

def test_empty_unless_statement_parenthesized_false
  unless(false)
  end
end

def test_empty_if_statement_true
  if true
  end
end

def test_empty_if_statement_false
  if false
  end
end

def test_empty_if_statement_parenthesized_true
  if(true)
  end
end

def test_empty_if_statement_parenthesized_false
  if(false)
  end
end