enum class Features {
    STALE_FLAG,
    ANOTHER_FLAG
}

class Sample {
    fun main() {
        if (featureService.isEnabled(STALE_FLAG)) {
            println("ok")
        }
        if (featureService.isEnabled(STALE_FLAG.key)) {
            println("ok")
        }
        if (featureService.isEnabled(Features.STALE_FLAG)) {
            println("ok")
        }
        if (featureService.isEnabled(Features.STALE_FLAG.key)) {
            println("ok")
        }
        if (featureService.isEnabled(PackageA.Features.STALE_FLAG)) {
            println("ok")
        }
        if (featureService.isEnabled(PackageA.Features.STALE_FLAG.key)) {
            println("ok")
        }
    }
}
