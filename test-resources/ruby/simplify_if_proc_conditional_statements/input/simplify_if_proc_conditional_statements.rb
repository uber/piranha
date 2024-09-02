after_commit :do_something, on: :create, if: Proc.new { true }
after_commit :do_lots_of_things, on: :create, if: Proc.new { false }

attribute :created_date, if: Proc.new { |object| true } do |object|
  object.actions
end

attribute :release_date, if: Proc.new { |object| false} do |object|
  object.actions
end

after_commit :do_something, on: :create, if: proc { true }
after_commit :do_lots_of_things, on: :create, if: proc { false }

attribute :planned_date, if: proc { |object| true } do |object|
  object.actions
end

attribute :future_date, if: proc { |object| false } do |object|
  object.actions
end
