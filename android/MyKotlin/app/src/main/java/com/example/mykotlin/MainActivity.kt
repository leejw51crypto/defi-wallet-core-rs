package com.example.mykotlin

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.widget.TextView
import com.example.mykotlin.databinding.ActivityMainBinding

public class MainActivity : AppCompatActivity() {

    private lateinit var binding: ActivityMainBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        // Example of a call to a native method
        binding.sampleText.text = stringFromJNI()
    }



    /**
     * A native method that is implemented by the 'mykotlin' native library,
     * which is packaged with this application.
     */
    external fun stringFromJNI(): String

    companion object {
        // Used to load the 'mykotlin' library on application startup.
        init {
            System.loadLibrary("mykotlin")
        }

        // write a function 1 integer parameter and return a integer
        @JvmStatic
        fun addOne(x: Int): Int {
            return x + 1
        }

        @JvmStatic
        fun readSecureStorage(key:String):  HashMap<String, String> {
            val myMap = HashMap<String, String>()
            myMap["key1"] = "value1"
            myMap["key2"] = "value2"
            return myMap
        }

    }
}