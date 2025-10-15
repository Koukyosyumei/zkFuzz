// import { InformationCircleIcon } from "@heroicons/react/20/solid";

export default function Example({
  text,
  linkName,
  link,
  color,
}: {
  text: string;
  linkName: string;
  link: string;
  color: "blue" | "pink";
}) {
  return (
    <div className={`rounded-md bg-${color}-500 bg-opacity-70 p-4 mb-10`}>
      <div className="flex">
        <div className="flex-shrink-0">
          {/* <InformationCircleIcon className="h-5 w-5 text-blue-400" aria-hidden="true" /> */}
        </div>
        <div className="flex-1 md:flex md:justify-between">
          <p className="text-sm">{text}</p>
          <p className="mt-3 text-sm md:mt-0 md:ml-6">
            <a href={link} target="_blank" rel="noreferrer" className="hover:text-gray-200 font-medium">
              {linkName}
              <span aria-hidden="true"> &rarr;</span>
            </a>
          </p>
        </div>
      </div>
    </div>
  );
}
