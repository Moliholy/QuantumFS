pragma solidity ^0.5.0;

contract QuantumFS
{
    mapping(address => string[]) private fileSystems;

    function currentRevision()
      public
      view
      returns (string memory, uint)
    {
        uint totalRevisions = totalRevisions();
        if (totalRevisions == 0) {
            return ("", 0);
        }

        uint lastRevision = totalRevisions - 1;
        return getRevision(lastRevision);
    }

    function totalRevisions()
      public
      view
      returns (uint)
    {
        return fileSystems[msg.sender].length;
    }

    function getRevision(uint _revision)
      public
      view
      returns (string memory, uint)
    {
        require(_revision > 0, "Revision number must be greater than zero");
        require(_revision <= totalRevisions(), "Invalid revision");
        return (fileSystems[msg.sender][_revision - 1], _revision);
    }

    function addRevision(string calldata _hash)
      external
    {
        fileSystems[msg.sender].push(_hash);
    }

    function evict()
      external
    {
        delete fileSystems[msg.sender];
    }
}
