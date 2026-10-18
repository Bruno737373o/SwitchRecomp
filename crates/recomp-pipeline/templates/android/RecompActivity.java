package org.switchrecomp.android;

import android.app.Activity;
import android.content.res.AssetManager;
import android.graphics.Color;
import android.graphics.Typeface;
import android.os.Bundle;
import android.util.Log;
import android.widget.ScrollView;
import android.widget.TextView;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;

public class RecompActivity extends Activity {
    private static final String TAG = "SwitchRecomp";
    private static final String ASSET_ROOT = "recomp";
    private static final Object LOCK = new Object();
    private static String result;

    static {
        System.loadLibrary("recomp_android");
    }

    private static native int nativeRun(String workDir, String logPath);

    private TextView output;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        output = new TextView(this);
        output.setTypeface(Typeface.MONOSPACE);
        output.setTextColor(Color.rgb(0xE6, 0xE6, 0xE6));
        output.setBackgroundColor(Color.rgb(0x14, 0x14, 0x1C));
        int pad = (int) (16 * getResources().getDisplayMetrics().density);
        output.setPadding(pad, pad, pad, pad);
        output.setText(getString(R.string.app_name) + "\nrunning...\n");
        ScrollView scroll = new ScrollView(this);
        scroll.setFillViewport(true);
        scroll.setBackgroundColor(Color.rgb(0x14, 0x14, 0x1C));
        scroll.addView(output);
        setContentView(scroll);

        new Thread(this::runOnce, "recomp-main").start();
    }

    private void runOnce() {
        String text;
        synchronized (LOCK) {
            if (result == null) {
                result = execute();
            }
            text = result;
        }
        final String shown = getString(R.string.app_name) + "\n\n" + text;
        runOnUiThread(() -> output.setText(shown));
    }

    private String execute() {
        File workDir = new File(getFilesDir(), ASSET_ROOT);
        File logFile = new File(getFilesDir(), "recomp.log");
        try {
            deleteRecursive(workDir);
            copyAssets(getAssets(), ASSET_ROOT, workDir);
        } catch (IOException err) {
            Log.e(TAG, "asset extraction failed", err);
            return "asset extraction failed: " + err;
        }
        int code = nativeRun(workDir.getAbsolutePath(), logFile.getAbsolutePath());
        String log = readFile(logFile);
        for (String line : log.split("\n")) {
            Log.i(TAG, line);
        }
        Log.i(TAG, "exit code " + code);
        return log + "\nexit code: " + code + "\n";
    }

    private static void copyAssets(AssetManager assets, String path, File dest) throws IOException {
        String[] children = assets.list(path);
        if (children == null || children.length == 0) {
            File parent = dest.getParentFile();
            if (parent != null && !parent.isDirectory() && !parent.mkdirs()) {
                throw new IOException("mkdir failed: " + parent);
            }
            try (InputStream in = assets.open(path); OutputStream out = new FileOutputStream(dest)) {
                byte[] buffer = new byte[16384];
                int read;
                while ((read = in.read(buffer)) != -1) {
                    out.write(buffer, 0, read);
                }
            }
            return;
        }
        if (!dest.isDirectory() && !dest.mkdirs()) {
            throw new IOException("mkdir failed: " + dest);
        }
        for (String child : children) {
            copyAssets(assets, path + "/" + child, new File(dest, child));
        }
    }

    private static void deleteRecursive(File file) {
        File[] children = file.listFiles();
        if (children != null) {
            for (File child : children) {
                deleteRecursive(child);
            }
        }
        file.delete();
    }

    private static String readFile(File file) {
        try (InputStream in = new FileInputStream(file)) {
            byte[] bytes = new byte[(int) file.length()];
            int offset = 0;
            while (offset < bytes.length) {
                int read = in.read(bytes, offset, bytes.length - offset);
                if (read < 0) {
                    break;
                }
                offset += read;
            }
            return new String(bytes, 0, offset, StandardCharsets.UTF_8);
        } catch (IOException err) {
            return "log unavailable: " + err;
        }
    }
}
