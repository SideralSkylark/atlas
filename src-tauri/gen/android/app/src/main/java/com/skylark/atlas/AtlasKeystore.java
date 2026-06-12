package com.skylark.atlas;

import android.app.Activity;
import android.content.Context;
import android.content.SharedPreferences;
import android.security.keystore.KeyGenParameterSpec;
import android.security.keystore.KeyProperties;
import android.util.Base64;

import java.security.KeyStore;
import java.util.Map;

import javax.crypto.Cipher;
import javax.crypto.KeyGenerator;
import javax.crypto.SecretKey;
import javax.crypto.spec.GCMParameterSpec;

import app.tauri.annotation.Command;
import app.tauri.annotation.InvokeArg;
import app.tauri.annotation.TauriPlugin;
import app.tauri.plugin.Invoke;
import app.tauri.plugin.JSArray;
import app.tauri.plugin.JSObject;
import app.tauri.plugin.Plugin;

@InvokeArg
class StoreSecretArgs {
  public String alias;
  public String value;
}

@InvokeArg
class AliasArgs {
  public String alias;
}

@TauriPlugin
public class AtlasKeystore extends Plugin {
  private static final String KEYSTORE_PROVIDER = "AndroidKeyStore";
  private static final String MASTER_KEY_ALIAS = "atlas_master_key";
  private static final String PREFS_NAME = "atlas_keystore_prefs";
  private static final String AES_MODE = "AES/GCM/NoPadding";

  private final Activity activity;

  public AtlasKeystore(Activity activity) {
    super(activity);
    this.activity = activity;
  }

  private SecretKey getOrCreateMasterKey() throws Exception {
    KeyStore keyStore = KeyStore.getInstance(KEYSTORE_PROVIDER);
    keyStore.load(null);

    if (keyStore.containsAlias(MASTER_KEY_ALIAS)) {
      KeyStore.SecretKeyEntry entry = (KeyStore.SecretKeyEntry) keyStore.getEntry(MASTER_KEY_ALIAS, null);
      return entry.getSecretKey();
    }

    KeyGenerator keyGenerator = KeyGenerator.getInstance(
        KeyProperties.KEY_ALGORITHM_AES, KEYSTORE_PROVIDER);
    keyGenerator.init(new KeyGenParameterSpec.Builder(
        MASTER_KEY_ALIAS,
        KeyProperties.PURPOSE_ENCRYPT | KeyProperties.PURPOSE_DECRYPT)
        .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
        .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
        .setKeySize(256)
        .build());
    return keyGenerator.generateKey();
  }

  @Command
  public void store_secret(Invoke invoke) {
    StoreSecretArgs args = invoke.parseArgs(StoreSecretArgs.class);

    if (args.alias == null || args.value == null) {
      invoke.reject("Alias and value must be provided");
      return;
    }

    try {
      SecretKey key = getOrCreateMasterKey();
      Cipher cipher = Cipher.getInstance(AES_MODE);
      cipher.init(Cipher.ENCRYPT_MODE, key);

      byte[] iv = cipher.getIV();
      byte[] ciphertext = cipher.doFinal(args.value.getBytes("UTF-8"));

      byte[] combined = new byte[iv.length + ciphertext.length];
      System.arraycopy(iv, 0, combined, 0, iv.length);
      System.arraycopy(ciphertext, 0, combined, iv.length, ciphertext.length);

      String encoded = Base64.encodeToString(combined, Base64.NO_WRAP);

      SharedPreferences prefs = activity.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
      prefs.edit().putString(args.alias, encoded).apply();

      invoke.resolve();
    } catch (Exception e) {
      invoke.reject(e.getMessage());
    }
  }

  @Command
  public void get_secret(Invoke invoke) {
    AliasArgs args = invoke.parseArgs(AliasArgs.class);

    if (args.alias == null) {
      invoke.reject("Alias must be provided");
      return;
    }

    SharedPreferences prefs = activity.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
    String encoded = prefs.getString(args.alias, null);

    if (encoded == null) {
      invoke.reject("Secret not found");
      return;
    }

    try {
      byte[] combined = Base64.decode(encoded, Base64.NO_WRAP);
      byte[] iv = new byte[12];
      byte[] ciphertext = new byte[combined.length - 12];
      System.arraycopy(combined, 0, iv, 0, 12);
      System.arraycopy(combined, 12, ciphertext, 0, ciphertext.length);

      SecretKey key = getOrCreateMasterKey();
      Cipher cipher = Cipher.getInstance(AES_MODE);
      cipher.init(Cipher.DECRYPT_MODE, key, new GCMParameterSpec(128, iv));

      String value = new String(cipher.doFinal(ciphertext), "UTF-8");
      JSObject res = new JSObject();
      res.put("value", value);
      invoke.resolve(res);
    } catch (Exception e) {
      invoke.reject(e.getMessage());
    }
  }

  @Command
  public void delete_secret(Invoke invoke) {
    AliasArgs args = invoke.parseArgs(AliasArgs.class);

    if (args.alias == null) {
      invoke.reject("Alias must be provided");
      return;
    }

    SharedPreferences prefs = activity.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
    if (prefs.contains(args.alias)) {
      prefs.edit().remove(args.alias).apply();
      invoke.resolve();
    } else {
      invoke.reject("Secret not found");
    }
  }

  @Command
  public void list_secrets(Invoke invoke) {
    try {
      SharedPreferences prefs = activity.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE);
      JSArray list = new JSArray();
      for (String key : prefs.getAll().keySet()) {
        list.put(key);
      }
      JSObject res = new JSObject();
      res.put("secrets", list);
      invoke.resolve(res);
    } catch (Exception e) {
      invoke.reject(e.getMessage());
    }
  }
}