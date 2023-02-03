package com.cronos.play

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.fragment.app.Fragment
import androidx.security.crypto.EncryptedFile
import androidx.security.crypto.MasterKey
import androidx.security.crypto.MasterKey.Builder
import java.io.File
import android.content.Context
import java.io.FileOutputStream
import java.io.FileInputStream


class SecureStorage {


    companion object {
        init {
        }



        @JvmStatic
        fun readSecureStorage(context:Context ,key:String):   HashMap<String, String> {

            val masterKey: MasterKey = Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build()

            val file = File(context.filesDir, key)
            val encryptedFile: EncryptedFile = EncryptedFile.Builder(
                context,
                file,
                masterKey,
                EncryptedFile.FileEncryptionScheme.AES256_GCM_HKDF_4KB
            ).build()

            val myMap = HashMap<String, String>()


            if (file.exists()) {
                encryptedFile.openFileInput().use { inputStream ->
                    var myvalue=inputStream.readBytes().toString(Charsets.UTF_8)
                    myMap["result"] = myvalue
                    myMap["success"]="1"
                    myMap["error"]=""
                }
            }
            else {
                myMap["result"] =""
                myMap["success"]="0"
                myMap["error"]="encrypt file not found"
            }

            return myMap
        }

        @JvmStatic
        fun writeSecureStorage(context:Context , key:String, value:String): Int {
            val masterKey: MasterKey = Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build()

            val file = File(context.filesDir, key)
            val encryptedFile: EncryptedFile = EncryptedFile.Builder(
                context,
                file,
                masterKey,
                EncryptedFile.FileEncryptionScheme.AES256_GCM_HKDF_4KB
            ).build()


            var ret=0; // 1: success, 0: fail
            if (file.exists()) {
                file.delete()
            }

            encryptedFile.openFileOutput().use { outputStream ->
                outputStream.write(value.toByteArray())
                ret=1;
            }


            return ret
        }

    }
}