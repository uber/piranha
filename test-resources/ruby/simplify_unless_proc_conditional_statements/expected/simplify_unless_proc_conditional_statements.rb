after_commit :do_lots_of_things, on: :create

attribute :release_date do |object|
  object.actions
end

after_commit :do_lots_of_other_things, on: :create

attribute :future_date do |object|
  object.actions
end
