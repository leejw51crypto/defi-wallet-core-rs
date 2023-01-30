

package com.example.myapplication
import kotlin.jvm.JvmName

@JvmName("SecureDiskRead")
fun SecureDiskRead(a: Int, b: Int): Int {
    return a + b
}

@JvmName("SecureDiskWrite")
fun SecureDiskWrite(a: Int, b: Int): Int {
    return a + b
}

class SecureDisk {

    fun read(a: Int, b: Int): Int {
        return a + b
    }

    fun write(a: Int, b: Int): Int {
        return a + b
    }
}




