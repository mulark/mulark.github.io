for file in $(find . -name '*.html')
do
    if [ $file != "./menu-boilerplate.html" ]; then
        echo $file
        sed '/<!-- begin menu block -->/,/<!-- end menu block -->/d' $file > ./temp
        cat ./menu-boilerplate.html ./temp > $file
        rm ./temp
    fi
done
