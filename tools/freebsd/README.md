# FreeBSD system integration

Agate ships with an rc.d script to start the service during system startup as a daemon and a corresponding newsyslog configuration to rotate logs. These instructions assume a default installation of `PREFIX=/usr/local` and the default settings in the rc.d script.

To configure and start Agate, follow these steps:

1.  Review the `/usr/local/etc/rc.d/agate` startup script and see if there are any settings you wish to configure. If so, make edits to `/etc/rc.conf` now.

1.  Enable the service:

    ```sh
    # sysrc agate_enable=YES
    ```

1.  Create `/usr/local/www/gemini/`, make it owned by `www:www`, and place your capsule in it.

    ```sh
    # mkdir /usr/local/www/gemini
    # chown www:www /usr/local/www/gemini
    # chmod 770 /usr/local/www/gemini
    ```

1.  Create `/usr/local/etc/agate/`, make it owned and writable by `www:www` so that Agate can manage certificates, and change the permissions to 770 for privacy.

    ```sh
    # mkdir /usr/local/etc/agate
    # chown www:www /usr/local/etc/agate
    # chmod 770 /usr/local/etc/agate
    ```

    You can change ownership back to `root` after a successful startup to forbid Agate from modifying the certificates, if you wish.

1.  Start the service:

    ```sh
    # service agate start
    ```
