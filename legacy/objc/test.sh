process() {
  SOURCE=$1
  FLAGNAME=$2
  FLAGTYPE=$3
  cp $SOURCE $SOURCE.bak
  ./piranha-objc.sh $SOURCE $FLAGNAME $FLAGTYPE > /dev/null 2> /dev/null
  CHANGES=$(diff -wB $SOURCE $SOURCE.expected)
  if [ "$CHANGES" != "" ]
  then
    echo "Expected refactoring is different from the generated refactoring using Piranha for "$SOURCE
    vimdiff $SOURCE $SOURCE.expected
  fi
  mv $SOURCE.bak $SOURCE

}

process tests/OptimisticNamed.m optimistic_stale_flag optimistic
process tests/OptimisticNamedOther.m optimistic_stale_flag optimistic
process tests/OptimisticNamedImpl.m optimistic_stale_flag optimistic
process tests/Treated.m UBExperimentNameSomething treated
process tests/Control.m UBExperimentNameSomething control

echo "Tests executed completely"
