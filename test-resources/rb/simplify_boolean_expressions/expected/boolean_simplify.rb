def test_boolean_not_true
    
end

def test_boolean_not_false
    do_something
end

def test_parenthesized_expression_true
    do_something
end

def test_parenthesized_expression_false
end

def test_boolean_something_and_true
    if abc()
        do_something
    end
end

def test_boolean_something_and_true_1
    if abc() && xyz()
        do_something
    end
end

def test_boolean_true_and_something
    if abc()
        do_something
    end
end

def test_boolean_true_and_something_1
    if abc() && xyz()
        do_something
    end
end

def test_boolean_something_and_false
end

def test_boolean_something_and_false_1
end

def test_boolean_false_and_something
end

def test_boolean_false_and_something_1
end

def test_boolean_something_or_true
    do_something
end

def test_boolean_something_or_true_1
    do_something
end

def test_boolean_true_or_something
    do_something
end

def test_boolean_true_or_something_1
    do_something
end

def test_boolean_something_or_false
    if abc()
        do_something
    end
end

def test_boolean_something_or_false_1
    if abc() || xyz()
        do_something
    end
end

def test_boolean_false_or_something
    if abc()
        do_something
    end
end

def test_boolean_false_or_something_1
    if abc() || xyz()
        do_something
    end
end