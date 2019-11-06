# Run this script after updating menu-boilerplate.html for any menuing changes to propogate those changes to all html files.
for file in $(find . -name '*.html')
do
    if [ $file != "./menu-boilerplate.html" ]; then
        echo $file
        sed '/<!-- begin menu block -->/,/<!-- end menu block -->/d' $file > ./temp
        cat ./menu-boilerplate.html ./temp > $file
        rm ./temp
    fi
done
