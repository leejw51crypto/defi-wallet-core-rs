package com.example.myapplication

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import androidx.fragment.app.Fragment
import androidx.security.crypto.EncryptedFile
import androidx.security.crypto.MasterKey
import androidx.security.crypto.MasterKey.Builder
import com.example.myapplication.databinding.FragmentFirstBinding
import java.io.File
import java.io.FileOutputStream
import java.io.FileInputStream
/**
 * A simple [Fragment] subclass as the default destination in the navigation.
 */
class FirstFragment : Fragment() {

    private var _binding: FragmentFirstBinding? = null

    // This property is only valid between onCreateView and
    // onDestroyView.
    private val binding get() = _binding!!

    override fun onCreateView(
        inflater: LayoutInflater, container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View? {

        _binding = FragmentFirstBinding.inflate(inflater, container, false)
        return binding.root

    }
    fun processWallet(i: Int){
      //  val myString = "TEST %d".format(i)
      //          Toast.makeText(getContext(), myString, Toast.LENGTH_SHORT).show()
        /*val mainKey = MasterKey.Builder(getContext()!!)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        val fileToWrite = File(DIRECTORY, "my_sensitive_data.txt")
        val encryptedFile = EncryptedFile.Builder(getContext()!!,
            fileToWrite,
            mainKey,
            EncryptedFile.FileEncryptionScheme.AES256_GCM_HKDF_4KB
        ).build()

        if (fileToWrite.exists()) {
            fileToWrite.delete()
        }

        val fileContent = "MY SUPER-SECRET INFORMATION"
            .toByteArray(StandardCharsets.UTF_8)
        encryptedFile.openFileOutput().apply {
            write(fileContent)
            flush()
            close()
        }*/

        var context=getContext()
        val masterKey: MasterKey = Builder(context!!)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()

        val file = File(context!!.filesDir, "secret_data")
        val encryptedFile: EncryptedFile = EncryptedFile.Builder(
            context,
            file,
            masterKey,
            EncryptedFile.FileEncryptionScheme.AES256_GCM_HKDF_4KB
        ).build()

        // write to the encrypted file        

        // write to the encrypted file
       // encryptedFile.openFileOutput().use { outputStream ->
       //     outputStream.write("MY SUPER-SECRET INFORMATION".toByteArray())
        //}
        // val encryptedOutputStream: FileOutputStream = encryptedFile.openFileOutput()
        
         if (file.exists()) {
             file.delete()
        }

         encryptedFile.openFileOutput().use { outputStream ->
            // get current time
            var currentTime = System.currentTimeMillis()
            // format string with currentTime
            var myString = "TESTCODE %d".format(currentTime)
             outputStream.write(myString.toByteArray())
        }
        
        
        // read the encrypted file

        // read the encrypted file
        //val encryptedInputStream: FileInputStream = encryptedFile.openFileInput()
        encryptedFile.openFileInput().use { inputStream ->
            var tmp=inputStream.readBytes().toString(Charsets.UTF_8)
            println(tmp)
            
        }


    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        binding.buttonFirst.setOnClickListener {
            //findNavController().navigate(R.id.action_FirstFragment_to_SecondFragment)

            for (i in 1..1 step 1) {
                this.processWallet(i)
            }


        }
    }

    override fun onDestroyView() {
        super.onDestroyView()
        _binding = null
    }
}