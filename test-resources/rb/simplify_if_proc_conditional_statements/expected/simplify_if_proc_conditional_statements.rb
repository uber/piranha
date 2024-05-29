after_commit :do_something, on: :create

attribute :created_date do |object|
  object.actions
end

after_commit :do_something, on: :create

attribute :planned_date do |object|
  object.actions
end
