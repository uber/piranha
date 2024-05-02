def test_boolean_not_true
    do_something if !true
end

def test_boolean_not_false
    do_something if !false
end

def test_parenthesized_expression_true
    do_something if (true)
end

def test_parenthesized_expression_false
    do_something if (false)
end

def test_boolean_something_and_true
    if abc() && true
        do_something
    end
end

def test_boolean_something_and_true_1
    if abc() && xyz() && true
        do_something
    end
end

def test_boolean_true_and_something
    if true && abc()
        do_something
    end
end

def test_boolean_true_and_something_1
    if true && abc() && xyz()
        do_something
    end
end

def test_boolean_something_and_false
    if abc() && false
        do_something
    end
end

def test_boolean_something_and_false_1
    if abc() && xyz() && false
        do_something
    end
end

def test_boolean_false_and_something
    if false && abc()
        do_something
    end
end

def test_boolean_false_and_something_1
    if false && abc() && xyz()
        do_something
    end
end

def test_boolean_something_or_true
    if abc() || true
        do_something
    end
end

def test_boolean_something_or_true_1
    if abc() || xyz() || true
        do_something
    end
end

def test_boolean_true_or_something
    if true || abc()
        do_something
    end
end

def test_boolean_true_or_something_1
    if true || abc() || xyz()
        do_something
    end
end

def test_boolean_something_or_false
    if abc() || false
        do_something
    end
end

def test_boolean_something_or_false_1
    if abc() || xyz() || false
        do_something
    end
end

def test_boolean_false_or_something
    if false || abc()
        do_something
    end
end

def test_boolean_false_or_something_1
    if false || abc() || xyz()
        do_something
    end
end