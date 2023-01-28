package com.example.mykotlin

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.fragment.app.Fragment
import androidx.security.crypto.EncryptedFile
import androidx.security.crypto.MasterKey
import androidx.security.crypto.MasterKey.Builder
//import com.example.mykotlin.databinding.FragmentFirstBinding
import java.io.File
import android.content.Context
import java.io.FileOutputStream
import java.io.FileInputStream


class SecureStorage {


    companion object {
        init {
        }



        @JvmStatic
        fun readSecureStorage(key:String):  HashMap<String, String> {
            val myMap = HashMap<String, String>()
            myMap["key1"] = "value1"
            myMap["key2"] = "value2"
            return myMap
        }

        @JvmStatic
        fun writeSecureStorage(context:Context , key:String, value:String): Int {

            //var context=ContextProvider.getActivity().getApplicationContext()
            val masterKey: MasterKey = Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build()

            val file = File(context.filesDir, "secret_data")
            val encryptedFile: EncryptedFile = EncryptedFile.Builder(
                context,
                file,
                masterKey,
                EncryptedFile.FileEncryptionScheme.AES256_GCM_HKDF_4KB
            ).build()


            if (file.exists()) {
                file.delete()
            }

            encryptedFile.openFileOutput().use { outputStream ->
                var currentTime = System.currentTimeMillis()
                var myString = "TESTCODE %d".format(currentTime)
                outputStream.write(myString.toByteArray())
            }



            return 100
        }

    }
}