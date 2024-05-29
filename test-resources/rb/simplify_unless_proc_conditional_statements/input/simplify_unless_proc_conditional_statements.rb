after_commit :do_something, on: :create, unless: Proc.new { true }
after_commit :do_lots_of_things, on: :create, unless: Proc.new { false }

attribute :created_date, unless: Proc.new { |object| true } do |object|
  object.actions
end

attribute :release_date, unless: Proc.new { |object| false} do |object|
  object.actions
end

after_commit :do_flag_enabled_stuff, on: :create, unless: proc { true }
after_commit :do_lots_of_other_things, on: :create, unless: proc { false }

attribute :planned_date, unless: proc { |object| true } do |object|
  object.actions
end

attribute :future_date, unless: proc { |object| false } do |object|
  object.actions
end
