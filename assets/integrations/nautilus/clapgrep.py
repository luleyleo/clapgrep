import os
import subprocess
from gi.repository import Nautilus, GObject, Gio


class ClapgrepMenuProvider(GObject.GObject, Nautilus.MenuProvider):
    def __init__(self):
        pass

    def __open_clapgrep(self, file):
        os.system(
            f"""flatpak run de.leopoldluley.Clapgrep "{file.get_location().get_path()}" &"""
        )

    def menu_activate_cb(self, menu, file):
        self.__open_clapgrep(file)

    def menu_background_activate_cb(self, menu, file):
        self.__open_clapgrep(file)

    def __create_sub_menu(self, file, additional):
        if file.get_file_type() == Gio.FileType.DIRECTORY:
            item = Nautilus.MenuItem(
                name="ClapgrepMenuProvider::Search::" + additional,
                label="Open in Clapgrep",
                tip="",
                icon="search-symbolic",
            )
            item.connect("activate", self.menu_activate_cb, file)

            return item

    def get_file_items(self, files):
        if len(files) == 1 and os.path.isdir(files[0]):
            item = self.__create_sub_menu(files[0], "File")

            return (item,)

    def get_background_items(self, file):
        item = self.__create_sub_menu(file, "Background")

        return (item,)
