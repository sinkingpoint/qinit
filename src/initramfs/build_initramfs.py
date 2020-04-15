import tempfile
import argparse
import pathlib
import shutil
import subprocess
import os.path
import os
import stat

def mkdir(base_dir, dir):
    pathlib.Path("{}{}".format(base_dir, dir)).mkdir(parents=True, exist_ok=True)

def main(bins, libs, init, output_file, compress):
    base_dir = tempfile.mkdtemp()
    mkdir(base_dir, "/bin")
    mkdir(base_dir, "/lib64")
    files = []
    files.append("./bin")
    files.append("./lib64")
    for exe in bins:
        shutil.copy(exe, base_dir+"/bin")
        os.chmod("{}/bin/{}".format(base_dir, os.path.basename(exe)), 0o755)
        files.append("./bin/{}".format(os.path.basename(exe)))
    
    for lib in libs:
        shutil.copy(lib, base_dir+"/lib64")
        os.chmod("{}/lib64/{}".format(base_dir, os.path.basename(lib)), 0o777)
        files.append("./lib64/{}".format(os.path.basename(lib)))
    
    shutil.copyfile(init, base_dir+"/init")
    os.chmod(base_dir+"/init", 0o755)
    files.append("./init")
    p = subprocess.Popen(['cpio', '-H', 'newc', '-o'], stdout=subprocess.PIPE, stdin=subprocess.PIPE, stderr=subprocess.PIPE, cwd=base_dir)
    output = p.communicate(input='\n'.join(files).encode('utf-8'))[0]
    if compress:
        p = subprocess.Popen(['gzip', '--best'], stdout=subprocess.PIPE, stdin=subprocess.PIPE, stderr=subprocess.PIPE)
        output = p.communicate(input=output)[0]

    with open(output_file, 'wb') as f:
        f.write(output)

    shutil.rmtree(base_dir)

    
if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Make an initramfs from a set of binaries/libraries')
    parser.add_argument('--bins', type=str, nargs='+', default=[], help='Files to go in /bin')
    parser.add_argument('--libs', type=str, nargs='+', default=[], help='Files to go in /lib64')
    parser.add_argument('--init', type=str, help='File to go in /init', required=True)
    parser.add_argument('--output', type=str, help='Filename to output', default='initramfs.cpio')
    parser.add_argument('--compress', help='Whether or not to compress the archive', action='store_true')
    args = parser.parse_args()
    main(args.bins, args.libs, args.init, args.output, args.compress)
