import { ChevronRightIcon, HomeIcon } from "@heroicons/react/24/solid";

const pages = [
  { name: "Projects", href: "#", current: false },
  { name: "Project Nero", href: "#", current: true },
  { name: "Material", href: "#", current: true },
];

export default function Breadcrumb() {
  return (
    <nav className="flex" aria-label="Breadcrumb">
      <ol
        role="list"
        className="flex select-none space-x-4 rounded-md bg-black px-6 shadow ring-2 ring-neutral-400"
      >
        <li className="flex">
          <div className="flex items-center">
            <a href="#" className="hover:text-white-500 text-neutral-400">
              <HomeIcon className="h-5 w-5 flex-shrink-0" aria-hidden="true" />
              <span className="sr-only">Home</span>
            </a>
          </div>
        </li>
        {pages.map((page) => (
          <li key={page.name} className="flex p-2">
            <div className="flex items-center text-neutral-500">
              <ChevronRightIcon
                className="h-5 w-5 flex-shrink-0 text-neutral-400"
                aria-hidden="true"
              />
              <a
                href={page.href}
                className="ml-4 text-sm font-medium text-neutral-500 hover:text-white"
                aria-current={page.current ? "page" : undefined}
              >
                {page.name}
              </a>
            </div>
          </li>
        ))}
      </ol>
    </nav>
  );
}
