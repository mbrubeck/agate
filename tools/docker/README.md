# Run agate in a docker container

(these instructions assume you use linux and have some experience with both docker and the command line)

## Building the image

### Clone the repository with git

I assume you have git already installed. If not, please search on how to do it in the internet.

```
git clone https://github.com/mbrubeck/agate
cd agate
```

### Build the image
Enter the `tools/docker` directory:

```
cd tools/docker
```
And now build the docker image:

```
docker build -t agate .
```

This process will take a few minutes because all the rust modules have to be compiled from source.


## start the docker container

```
docker run -t -d --name agate -p 1965:1965 -v /var/www/gmi:/gmi -v /var/www/gmi/.certificates:/app/.certificates -e HOSTNAME=example.org -e LANG=en-US agate:latest
```

You have to replace `/var/www/gmi/` with the folder where you'd like to have gemtext files and `/var/www/gmi/.certificates/` with the folder where you'd like to have your certificates stored. You also have to have to replace `example.org` with your domain name and if plan to speak in a different language than english in your gemini space than you should replace `en-US` with your countries language code (for example de-DE or fr-CA).

## That's it! Now have agate running in a docker container!

Just open a gemini browser and point to you server